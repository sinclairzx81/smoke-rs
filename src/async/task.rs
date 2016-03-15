/*--------------------------------------------------------------------------

smoke-rs

The MIT License (MIT)

Copyright (c) 2015 Haydn Paterson (sinclair) <haydn.developer@gmail.com>

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in
all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
THE SOFTWARE.

---------------------------------------------------------------------------*/

use std::sync::mpsc::{sync_channel, SyncSender};
use std::sync::mpsc::{ SendError, RecvError };
use std::thread;
use std::thread::JoinHandle;

//---------------------------------------------------------
// TaskFunc
//---------------------------------------------------------

trait TaskFunc<T> {
    type Output;
    fn call(self: Box<Self>, value:T) -> Self::Output;
}
impl<R, T, F: FnOnce(T) -> R> TaskFunc<T> for F {
    type Output = R;
    fn call(self: Box<Self>, value: T) -> R {
        self(value)
    }
}

//---------------------------------------------------------
// TaskSender<T> 
//---------------------------------------------------------
pub struct TaskSender<T> {
   sender: SyncSender<T>
}
impl<T> TaskSender<T> {
    fn new(sender: SyncSender<T>) -> TaskSender<T> {
      TaskSender {
        sender: sender
      }
    }
    pub fn send(self, value:T) -> Result<(), SendError<T>> {
      self.sender.send(value)
    }
}

//---------------------------------------------------------
// Task<T, E> 
//---------------------------------------------------------
pub struct Task<T> {
    func: Box<TaskFunc<TaskSender<T>, Output = Result<(), SendError<T>>> + Send + 'static>
}
impl <T> Task<T> {
    
    //---------------------------------------------------------
    // new() creates a new task.
    //---------------------------------------------------------   
    pub fn new<F>(func: F) -> Task<T>
        where F: FnOnce(TaskSender<T>) -> Result<(), SendError<T>> + Send + 'static {
        Task {
            func: Box::new(func)
        }
    }
    
    //---------------------------------------------------------
    // sync() executes this task synchronously.
    //---------------------------------------------------------     
    pub fn sync(self) -> Result<T, RecvError> {
        let (sender, receiver) = sync_channel(1);
        let sender = TaskSender::new(sender);
        match self.func.call(sender) {
            Err(_) => panic!("task failed to execute."),
            Ok(_)  => receiver.recv()
        }
    }
}

//---------------------------------------------------------
// Task<T, E> for Send + 'static
//---------------------------------------------------------
impl<T> Task<T> where T: Send + 'static {
    
    //---------------------------------------------------------
    // async() executes this task asynchronously.
    //--------------------------------------------------------- 
    pub fn async(self) -> JoinHandle<T> {
        thread::spawn(move || self.sync().unwrap())
    }
    
    //---------------------------------------------------------
    // then() continues this task into another.
    //---------------------------------------------------------
    pub fn then<U, F>(self, func: F) -> Task<U> where 
        U : Send + 'static,
        F : FnOnce(T) -> Task<U> + Send + 'static {
      func(self.sync().unwrap())
    }
    
    //---------------------------------------------------------
    // all() runs the given tasks in parallel.
    //---------------------------------------------------------
    pub fn all(tasks: Vec<Task<T>>) -> Task<Vec<T>> {
        Task::<Vec<T>>::new(move |sender| {
            let result:Result<Vec<_>, RecvError> 
                = tasks.into_iter()
                       .map(|task| thread::spawn(|| task.sync() ) )
                       .collect::<Vec<_>>()
                       .into_iter()
                       .map(|handle| handle.join().unwrap())
                       .collect();
            match result {
              Ok (value) => sender.send(value),
              Err(error) => panic!(error)
            }
        })
    }
}