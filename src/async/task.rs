/*--------------------------------------------------------------------------
// smoke-rs
//
// The MIT License (MIT)
//
// Copyright (c) 2016 Haydn Paterson (sinclair) <haydn.developer@gmail.com>
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
// THE SOFTWARE.
//
// ---------------------------------------------------------------------------*/

use std::sync::mpsc::{
  SyncSender, 
  SendError, 
  RecvError
};
use super::scheduling::{
  WaitHandle,
  Scheduler,
  SyncScheduler,
  ThreadScheduler,
  ThreadPoolScheduler
};

/// TaskSender<T>
/// 
/// Wraps a mpsc SyncSender to enforce only single send. The
/// task sender achieves this by capturing itself on send vs
/// the SyncSender allowing for multiple sends.
pub struct TaskSender<T> {
   sender: SyncSender<T>
}
impl<T> TaskSender<T>  {
    /// creates a new TaskSender<T>. 
    pub fn new(sender: SyncSender<T>) -> TaskSender<T> {
      TaskSender { sender: sender }
    }
    
    /// The send() function will resolve this task.
    pub fn send(self, value:T) -> Result<(), SendError<T>> {
      self.sender.send(value)
    }
}

/// TaskFunc<T>
pub trait TaskFunc<T> {
    type Output;
    fn call(self: Box<Self>, value:T) -> Self::Output;
}
impl<R, T, F: FnOnce(T) -> R> TaskFunc<T> for F {
    type Output = R;
    fn call(self: Box<Self>, value: T) -> R {
        self(value)
    }
}

/// Task<T>
pub struct Task<T> {
    pub func: Box<TaskFunc<TaskSender<T>, Output = Result<(), SendError<T>>> + Send + 'static>
}
impl <T> Task<T> where T: Send + 'static {
    /// new() creates a new task.
    pub fn new<F>(func: F) -> Task<T> 
      where F: FnOnce(TaskSender<T>) -> Result<(), SendError<T>> + Send + 'static {
        Task { func: Box::new(func) }
    }
    
    /// The map() function allows the caller to transform the result of
    /// a task into another type.
    pub fn map<U, F>(self, func: F) -> Task<U> where 
        U : Send + 'static,
        F : FnOnce(Result<T, RecvError>) -> U + Send + 'static {
          Task::<U>::new(move |sender| {
              let result = ThreadScheduler.run(self).wait();
              sender.send(func(result))
          })
    }
    
    /// The then() functions allows the caller to chain consecutive
    /// tasks. This function returns a new Task to be waited on later.
    pub fn then<U, F>(self, func: F) -> Task<U> where 
        U : Send + 'static,
        F : FnOnce(Result<T, RecvError>) -> Task<U> + Send + 'static {
          Task::new(move |sender| {
            let result_self  = ThreadScheduler.run(self).wait();
            let result_other = ThreadScheduler.run(func(result_self)).wait();
            sender.send(result_other.unwrap())
          })
    }
    
    /// The all() function will return a task which will process
    /// the given tasks in parallel. The caller may provide the 
    /// number of threads the task should use to process each 
    /// task. 
    pub fn all(threads: usize, tasks: Vec<Task<T>>) -> Task<Vec<T>>  {
        Task::<Vec<T>>::new(move |sender| {
              let scheduler = ThreadPoolScheduler::new(threads);
              let result    = tasks.into_iter()
                                .map(|task| scheduler.run(task))
                                .collect::<Vec<_>>()
                                .into_iter()
                                .map(|handle| handle.wait())
                                .collect::<Result<Vec<_>, RecvError>>();          
            match result {
              Ok (value) => sender.send(value),
              Err(error) => panic!(error)
            }
        })
    }
    
    /// The schedule() function will begin the task on the given 
    /// scheduler. The function will return a wait handle to the
    /// caller.
    pub fn schedule<S: Scheduler>(self, scheduler:S) -> WaitHandle<T> {
        scheduler.run(self)
    }
    

    /// The async() function will begin this task on its own thread.
    /// The result will be passed into the given closure. This 
    /// function will return a wait handle to the caller. The caller
    /// may wait on the result of the closure.
    pub fn async<U, F>(self, func: F) -> WaitHandle<U>
        where U : Send + 'static,
              F : FnOnce(Result<T, RecvError>) -> U + Send + 'static {
        ThreadScheduler.run(Task::new(|sender| {
          let result    = ThreadScheduler.run(self).wait();
          let result    = func(result);
          sender.send(result)
        }))
    }
    

    /// The wait() function will begin the task on the current 
    /// thread, causing the current thread to block until the 
    /// task completes.
    pub fn wait(self) -> Result<T, RecvError> {
        SyncScheduler.run(self).wait()
    }
}

impl Task<()> {
    /// Creates a Task<T> that will block for the given duration.
    pub fn delay(millis: u64) -> Task<()> {
      use std::thread;
      use std::time::Duration;
      Task::new(move|sender| {
        thread::sleep(Duration::from_millis(millis));
        sender.send(())
      })
    }
}

