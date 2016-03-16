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

use std::sync::mpsc::{ sync_channel, SyncSender};
use std::sync::mpsc::{ SendError, RecvError };
use super::scheduler::{ Scheduler, Handle};

///---------------------------------------------------------
/// GLOBAL_SCHEDULER: The default task scheduler (32)
///---------------------------------------------------------
lazy_static! {
    static ref GLOBAL_SCHEDULER: Scheduler = {
      Scheduler::new(32)
    };
}

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
    scheduler : Scheduler,
    func      : Box<TaskFunc<TaskSender<T>, Output = Result<(), SendError<T>>> + Send + 'static>
}
impl <T> Task<T> where T: Send + 'static {
    
    //---------------------------------------------------------
    // scheduled() creates a tasked on this scheduler.
    //---------------------------------------------------------   
    pub fn scheduled<F>(scheduler: Scheduler, func: F) -> Task<T>
        where F: FnOnce(TaskSender<T>) -> Result<(), SendError<T>> + Send + 'static {
        Task {
            scheduler: scheduler,
            func     : Box::new(func)
        }
    }
    
    //---------------------------------------------------------
    // new() creates a scheduled task.
    //---------------------------------------------------------  
    pub fn new<F>(func: F) -> Task<T> 
      where F: FnOnce(TaskSender<T>) -> Result<(), SendError<T>> + Send + 'static {
        Task {
            scheduler: GLOBAL_SCHEDULER.clone(),
            func     : Box::new(func)
        }
    }
    
    //---------------------------------------------------------
    // app() creates a scheduled task.
    //--------------------------------------------------------- 
    pub fn all(tasks: Vec<Task<T>>) -> Task<Vec<T>> {
        Task::<Vec<T>>::new(move |sender| {
            let result:Result<Vec<_>, RecvError> 
                = tasks.into_iter()
                       .map(|task| task.async(|result| result))
                       .collect::<Vec<_>>()
                       .into_iter()
                       .map(|handle| handle.wait().unwrap())
                       .collect();
            match result {
              Ok (value) => sender.send(value),
              Err(error) => panic!(error)
            }
        })
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
    
    //---------------------------------------------------------
    // async() executes this task asynchronously.
    //--------------------------------------------------------- 
    pub fn async<F, U>(self, func: F) -> Handle<U>
        where U: Send + 'static,
              F: FnOnce(Result<T, RecvError>) -> U + Send + 'static {
        let scheduler = self.scheduler.clone();
        scheduler.run(move || func(self.sync()))
    }
    
    //---------------------------------------------------------
    // then() continues this task into another.
    //---------------------------------------------------------
    pub fn then<U, F>(self, func: F) -> Task<U> where 
        U : Send + 'static,
        F : FnOnce(T) -> Task<U> + Send + 'static {
          Task::<U>::new(move |sender| {
             let result = func(self.sync().unwrap());
             sender.send(result.sync().unwrap())
          })
    } 
}

//------------------------------------------
// extensions
//------------------------------------------
use std::thread;
use std::time::Duration;

impl Task<()> {
    //---------------------------------------------------------
    // delay() creates a delay with the given timeout.
    //---------------------------------------------------------
    pub fn delay(millis: u64) -> Task<()> {
      Task::new(move |sender| {
        thread::sleep(Duration::from_millis(millis));
        sender.send(())
      })
    }
}