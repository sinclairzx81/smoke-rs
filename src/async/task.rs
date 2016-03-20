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

/// A container for a SyncSender&lt;T&gt; to enforce single send.
pub struct TaskSender<T> {
   sender: SyncSender<T>
}
impl<T> TaskSender<T>  {
    /// Creates a new task sender.
    pub fn new(sender: SyncSender<T>) -> TaskSender<T> {
      TaskSender { sender: sender }
    }
    /// Resolves this task sender with the given value.
    pub fn send(self, value:T) -> Result<(), SendError<T>> {
      self.sender.send(value)
    }
}

/// Specialized boxed FnOnce() closure type for tasks.
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

/// Encapsulates an asynchronous operation. Tasks can be run either synchronously or asynchronously.
pub struct Task<T> {
    /// The closure to resolve this task.
    pub func: Box<TaskFunc<TaskSender<T>, Output = Result<(), SendError<T>>> + Send + 'static>
}
impl <T> Task<T> where T: Send + 'static {
    /// Creates a new task.
    /// # Example
    /// ```
    /// use smoke::async::Task;
    ///
    /// let task = Task::new(|sender| sender.send("hello"));
    /// ```    
    pub fn new<F>(func: F) -> Task<T> 
      where F: FnOnce(TaskSender<T>) -> Result<(), SendError<T>> + Send + 'static {
        Task { func: Box::new(func) }
    }
    
    /// Maps this task into another value.
    /// # Example
    /// ```
    /// use smoke::async::Task;
    ///
    /// let task = Task::new(|sender| sender.send("hello"))
    ///                 .map(|n| 10);
    /// assert_eq!(task.wait().unwrap(), 10);
    /// ```       
    pub fn map<U, F>(self, func: F) -> Task<U> where 
        U : Send + 'static,
        F : FnOnce(Result<T, RecvError>) -> U + Send + 'static {
          Task::<U>::new(move |sender| {
              let result = ThreadScheduler.run(self).wait();
              sender.send(func(result))
          })
    }
    
    /// Creates a new task that runs this task followed by the next.
    /// # Example
    /// ```
    /// use smoke::async::Task;
    ///
    /// fn add(a: i32, b: i32) -> Task<i32> {
    ///   Task::new(move |sender| sender.send(a + b)) 
    /// }
    ///
    /// let task = add(10, 20).then(|result| add(result.unwrap(), 20));
    /// assert_eq!(task.wait().unwrap(), 50);
    /// ```  
    pub fn then<U, F>(self, func: F) -> Task<U> where 
        U : Send + 'static,
        F : FnOnce(Result<T, RecvError>) -> Task<U> + Send + 'static {
          Task::new(move |sender| {
            let result_self  = ThreadScheduler.run(self).wait();
            let result_other = ThreadScheduler.run(func(result_self)).wait();
            sender.send(result_other.unwrap())
          })
    }
    
    /// Creates a new task that will process the given tasks in
    /// parallel. Tasks executed in parallel will be scheduled
    /// on a internal threadpool with a pool size of the threads
    /// argument.
    /// # Example
    /// ```
    /// use smoke::async::Task;
    ///
    /// fn add(a: i32, b: i32) -> Task<i32> {
    ///   Task::new(move |sender| sender.send(a + b)) 
    /// }
    ///
    /// let task = Task::all(4, vec![
    ///   add(1, 2), 
    ///   add(3, 4), 
    ///   add(5, 6), 
    ///   add(7, 8)
    /// ]);
    /// ```      
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
    
    /// Schedules this task to run on the given scheduler. Returns
    /// a wait handle to the caller.
    /// # Example
    /// ```
    /// use smoke::async::Task;
    /// use smoke::async::ThreadPoolScheduler;
    ///
    /// fn add(a: i32, b: i32) -> Task<i32> {
    ///   Task::new(move |sender| sender.send(a + b)) 
    /// }
    ///
    /// let scheduler = ThreadPoolScheduler::new(4);
    /// let handle = add(10, 20).schedule(scheduler);
    /// assert_eq!(handle.wait().unwrap(), 30); 
    /// ```     
    pub fn schedule<S: Scheduler>(self, scheduler:S) -> WaitHandle<T> {
        scheduler.run(self)
    }
    
    /// Runs this task immediately on its only thread. The result will
    /// be passed into the given closure.
    /// # Example
    /// ```
    /// use smoke::async::Task;
    ///
    /// fn add(a: i32, b: i32) -> Task<i32> {
    ///   Task::new(move |sender| sender.send(a + b)) 
    /// }
    ///
    /// let handle = add(10, 20).async(|result| {
    ///    assert_eq!(result.unwrap(), 30);
    ///    123  
    /// });
    /// assert_eq!(handle.wait().unwrap(), 123);
    /// ```     
    pub fn async<U, F>(self, func: F) -> WaitHandle<U>
        where U : Send + 'static,
              F : FnOnce(Result<T, RecvError>) -> U + Send + 'static {
        ThreadScheduler.run(Task::new(|sender| {
          let result    = ThreadScheduler.run(self).wait();
          let result    = func(result);
          sender.send(result)
        }))
    }
    
    /// Waits synchronously for this task to complete.
    /// # Example
    /// ```
    /// use smoke::async::Task;
    ///
    /// let task = Task::new(|sender| sender.send(10));
    /// assert_eq!(task.wait().unwrap(), 10);
    /// ```      
    pub fn wait(self) -> Result<T, RecvError> {
        SyncScheduler.run(self).wait()
    }
}

impl Task<()> {
    /// Creates a task that will delay for the given duration
    /// in milliseconds.
    /// # Example
    /// ```
    /// use smoke::async::Task;
    ///
    /// Task::delay(1000).wait();
    /// ```      
    pub fn delay(millis: u64) -> Task<()> {
      use std::thread;
      use std::time::Duration;
      Task::new(move|sender| {
        thread::sleep(Duration::from_millis(millis));
        sender.send(())
      })
    }
}

