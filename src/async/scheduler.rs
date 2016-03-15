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

use std::thread;
use std::thread::JoinHandle as StdJoinHandle;
use std::sync::{Arc, Mutex, Condvar};
use threadpool::ThreadPool;
use std::any::Any;

// future:
// use std::panic::recover;

///-------------------------------------------
/// ThreadJoinHandle<T> 
///-------------------------------------------
#[derive(Clone)]
struct ThreadJoinHandle<T> {
    handle: Arc<Mutex<Option<StdJoinHandle<T>>>>
}
impl<T> ThreadJoinHandle<T> {
  fn new(handle: Arc<Mutex<Option<StdJoinHandle<T>>>>) -> ThreadJoinHandle<T> {
    ThreadJoinHandle {
      handle: handle
    }
  }
  pub fn join(self) -> Result<T, Box<Any + Send + 'static>> {
    let mut option = self.handle.lock().unwrap();
    match option.take() {
      Some(handle) => handle.join(),
      None => panic!()
    }
  }
}
///-------------------------------------------
/// ThreadPoolJoinHandle<T> 
///-------------------------------------------
#[derive(Clone)]
struct ThreadPoolJoinHandle<T> {
    handle: Arc<(Mutex<Option<T>>, Condvar)>
}
impl<T> ThreadPoolJoinHandle<T> {
  fn new(handle: Arc<(Mutex<Option<T>>, Condvar)>) -> ThreadPoolJoinHandle<T> {
      ThreadPoolJoinHandle {
        handle: handle
      }
    }
  ///-------------------------------------------
  /// join() joins this thread back on the caller thread.
  ///-------------------------------------------  
  fn join(self) -> Result<T, Box<Any + Send + 'static>> {
    let &(ref lock, ref cvar) = &*self.handle;
    let mut value = lock.lock().unwrap();
    while value.is_none() {
        value = cvar.wait(value).unwrap();
    }
    match value.take() {
      Some(n) => Ok(n),
      None => Err(Box::new("no result."))
    }
  }
}
///-------------------------------------------
/// JoinHandleOption<T> 
///-------------------------------------------
#[derive(Clone)]
enum JoinHandleOption<T> {
  Thread(ThreadJoinHandle<T>),
  ThreadPool(ThreadPoolJoinHandle<T>)
}

///-------------------------------------------
/// JoinHandleOption<T> 
///-------------------------------------------
#[derive(Clone)]
pub struct JoinHandle<T> {
  option: JoinHandleOption<T>
}
impl<T> JoinHandle<T> {
  fn new(option: JoinHandleOption<T>) -> JoinHandle<T> {
    JoinHandle {
      option: option
    }
  }
  ///-------------------------------------------
  /// join() joins this thread back on the caller thread.
  ///-------------------------------------------    
  pub fn join(self) -> Result<T, Box<Any + Send + 'static>> {
    match self.option {
      JoinHandleOption::Thread(handle) => handle.join(),
      JoinHandleOption::ThreadPool(handle) => handle.join()
    }
  }
}

///-------------------------------------------
/// ThreadScheduler<T> 
///-------------------------------------------
#[derive(Clone)]
struct ThreadScheduler;
impl ThreadScheduler {
  ///---------------------------------------------
  /// run(): queues work on this scheduler.
  ///---------------------------------------------   
  fn run<F, T>(&self, f: F) -> JoinHandle<T>
  where T: Send + 'static,
        F: FnOnce() -> T + Send + 'static {
    let option = JoinHandleOption::Thread (
      ThreadJoinHandle::new(Arc::new(Mutex::new(Some(thread::spawn(f)))))
    ); JoinHandle::new(option)
  }
}

///-------------------------------------------
/// ThreadPoolScheduler<T> 
///-------------------------------------------
#[derive(Clone)]
struct ThreadPoolScheduler {
  threadpool: ThreadPool
}
impl ThreadPoolScheduler {
  fn new(bound:usize) -> ThreadPoolScheduler {
    ThreadPoolScheduler {
      threadpool: ThreadPool::new(bound)
    }
  }
  ///---------------------------------------------
  /// run(): queues work on this scheduler.
  ///---------------------------------------------   
  fn run<F, T>(&self, f: F) -> JoinHandle<T> where 
    T: Send + 'static,
    F: FnOnce() -> T + Send + 'static {
    let handle   = Arc::new((Mutex::new(None), Condvar::new()));
    let clone    = handle.clone();
    self.threadpool.execute(move || {
        // WARNING: panics in this thread can't be
        // caught until std::panic::recover becomes
        // stable.
        let result = f();
        
        let &(ref lock, ref cvar) = &*clone;
        let mut value = lock.lock().unwrap();
        *value = Some(result);
        cvar.notify_one();  
    });
    let option = JoinHandleOption::ThreadPool (
      ThreadPoolJoinHandle::new(handle)
    ); JoinHandle::new(option)
  }
}

///-------------------------------------------
/// SchedulerOption<T> 
///-------------------------------------------
#[derive(Clone)]
enum SchedulerOption {
  Thread(ThreadScheduler),
  ThreadPool(ThreadPoolScheduler)
}

///-------------------------------------------
/// Scheduler
///-------------------------------------------
#[derive(Clone)]
pub struct Scheduler {
  option: SchedulerOption
}
impl Scheduler {
  
  ///---------------------------------------------
  /// thread(): thread based scheduler.
  ///--------------------------------------------- 
  pub fn thread() -> Scheduler {
    Scheduler {
      option: SchedulerOption::Thread(ThreadScheduler)
    }
  }
  
  ///---------------------------------------------
  /// threadpool(): threadpool based scheduler.
  ///---------------------------------------------
  pub fn threadpool(threads: usize) -> Scheduler {
    Scheduler {
      option: SchedulerOption::ThreadPool(ThreadPoolScheduler::new(threads))
    }
  }
  
  ///---------------------------------------------
  /// run(): queues work on this scheduler.
  ///--------------------------------------------- 
  pub fn run<F, T>(&self, f: F) -> JoinHandle<T> where 
    T: Send + 'static,
    F: FnOnce() -> T + Send + 'static {
    match self.option {
      SchedulerOption::Thread(ref scheduler) 
        => scheduler.run(f),
      SchedulerOption::ThreadPool(ref scheduler) 
        => scheduler.run(f)
    }
  }
}
