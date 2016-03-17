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
/// DEFAULT_TASK_SCHEDULER
///
/// A default Task scheduler used by Tasks created without
/// a specified scheduler. 
///---------------------------------------------------------
extern crate num_cpus;
lazy_static! {
    static ref DEFAULT_TASK_SCHEDULER: Scheduler = {
      match num_cpus::get() {
        0 ... 7 => Scheduler::new(8),
        cpus    => Scheduler::new(cpus)
      }
    };
}

///---------------------------------------------------------
/// TaskFunc<T>
///
/// Specialized FnOnce closure for Task<T>. Provides a boxable
/// FnOnce signature for Task resolution, and acts as a fill in 
/// for a possible Box<FnOnce> capability in future. 
///---------------------------------------------------------
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

///---------------------------------------------------------
/// TaskSender<T>
///
/// Passed in on Task creation, the TaskSender<T> is responsible
/// for resolving a value for a given task. The TaskSender
/// cannot be cloned but instead be 'moved' into additional
/// scopes (such moving the sender to a thread::spawn), and 
/// can only be resolved once by the caller calling .send()
/// which captures ownership of the sender.
///---------------------------------------------------------
pub struct TaskSender<T> {
   sender: SyncSender<T>
}
impl<T> TaskSender<T>  {
    fn new(sender: SyncSender<T>) -> TaskSender<T> {
      TaskSender { sender: sender }
    }
    ///---------------------------------------------------------
    /// The send() function will resolve the task in which this
    /// sender originated.
    ///---------------------------------------------------------
    pub fn send(self, value:T) -> Result<(), SendError<T>> {
      self.sender.send(value)
    }
}

///---------------------------------------------------------
/// Task<T>
///
/// A Task encapsulates a synchronous code block which yields 
/// a single result. Callers can use a task to wrap existing 
/// synchronous logic in which the task provides options for 
/// synchronous/asynchronous or parallel execution of that 
/// block.
///---------------------------------------------------------
pub struct Task<T> {
    scheduler : Scheduler,
    func      : Box<TaskFunc<TaskSender<T>, Output = Result<(), SendError<T>>> + Send + 'static>
}
impl <T> Task<T> where T: Send + 'static {
    
    ///---------------------------------------------------------
    /// The scheduled() function will create a new task that
    /// will be scheduled on the given task given task scheduler.
    ///---------------------------------------------------------
    pub fn scheduled<F>(scheduler: Scheduler, func: F) -> Task<T>
        where F: FnOnce(TaskSender<T>) -> Result<(), SendError<T>> + Send + 'static {
        Task {
            scheduler: scheduler,
            func     : Box::new(func)
        }
    }
    
    ///---------------------------------------------------------
    /// The new() function will create a new task on the global
    /// task scheduler. 
    ///---------------------------------------------------------  
    pub fn new<F>(func: F) -> Task<T> 
      where F: FnOnce(TaskSender<T>) -> Result<(), SendError<T>> + Send + 'static {
        Task::scheduled(DEFAULT_TASK_SCHEDULER.clone(), func)
    }
    
    ///-----------------------------------------------------------
    /// The then() function will execute the given task after
    /// the task in which its applied and returns a new task
    /// to the caller.
    ///-----------------------------------------------------------
    pub fn then<U, F>(self, func: F) -> Task<U> where 
        U : Send + 'static,
        F : FnOnce(Result<T, RecvError>) -> Task<U> + Send + 'static {
          Task::<U>::new(move |sender| {
             let result = func(self.wait());
             sender.send(result.wait().unwrap())
          })
    }
    
    ///-----------------------------------------------------------
    /// The all() method returns a Task<Vec<T>> that resolves 
    /// when all of the tasks in the enumerable argument have 
    /// resolved. If error, this task will resolve the first Err 
    /// found, otherwise will resolve results as Vec<T>.
    ///------------------------------------------------------------ 
    pub fn all(tasks: Vec<Task<T>>) -> Task<Vec<T>>  {
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
    
    ///---------------------------------------------------------
    /// The wait() function will block the current thread to
    /// obtain the result.
    ///---------------------------------------------------------     
    pub fn wait(self) -> Result<T, RecvError> {
        let (sender, receiver) = sync_channel(1);
        let sender = TaskSender::new(sender);
        match self.func.call(sender) {
            Err(_) => panic!("task failed to execute."),
            Ok(_)  => receiver.recv()
        }
    }
    
    ///-----------------------------------------------------------
    /// Runs async() function runs this task asynchronously and 
    ///  returns a handle for the caller to wait on. When this 
    /// task resolves, the result will be pushed into the
    /// completion closure. The closure may opt to return a 
    /// result which will be available when waiting on the handle.
    ///----------------------------------------------------------- 
    pub fn async<U, F>(self, func: F) -> Handle<U>
        where U : Send + 'static,
              F : FnOnce(Result<T, RecvError>) -> U + Send + 'static {
        let scheduler = self.scheduler.clone();
        scheduler.run(move || func(self.wait()))
    }
}
impl Task<()> {
    
    ///---------------------------------------------------------
    /// The delay() function creates a simple delayed task. The 
    /// delay itself amounts to a thread::sleep on the default
    /// task scheduler. 
    ///---------------------------------------------------------
    pub fn delay(millis: u64) -> Task<()> {
      use std::thread;
      use std::time::Duration;
      Task::new(move|sender| {
        thread::sleep(Duration::from_millis(millis));
        sender.send(())
      })
    }
}