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

extern crate threadpool;

use self::threadpool::ThreadPool;
use std::sync::{Arc, Mutex, Condvar};
use std::any::Any;

///-------------------------------------------
/// Handle<T> 
///-------------------------------------------
pub struct Handle<T> {
    handle: Arc<(Mutex<Option<T>>, Condvar)>
}
impl<T> Handle<T> {
  fn new(handle: Arc<(Mutex<Option<T>>, Condvar)>) -> Handle<T> {
    Handle {
      handle: handle
    }
  }
  pub fn wait(self) -> Result<T, Box<Any + Send + 'static>> {
    let &(ref lock, ref cvar) = &*self.handle;
    let mut value = lock.lock().unwrap();
    while value.is_none() {
        value = cvar.wait(value).unwrap();
    }
    match value.take() {
      Some(n) => Ok(n),
      None    => Err(Box::new("no result."))
    }
  } 
}
///-------------------------------------------
/// Scheduler
///-------------------------------------------
#[derive(Clone)]
pub struct Scheduler {
  threadpool: Arc<Mutex<ThreadPool>>
}
impl Scheduler {
  pub fn new(threads:usize) -> Scheduler {
    Scheduler {
      threadpool: Arc::new(Mutex::new(ThreadPool::new(threads)))
    }
  }
  pub fn run<T: Send + 'static, F:FnOnce() -> T + Send + 'static>(&self, func: F) -> Handle<T> {
    let handle     = Arc::new((Mutex::new(None), Condvar::new()));
    let clone      = handle.clone();
    let threadpool = self.threadpool.lock().unwrap();
    threadpool.execute(move || {
        // WARNING: panics in this thread can't be caught
        // until std::panic::recover becomes stable.
        let result = func();
        let &(ref lock, ref cvar) = &*clone;
        let mut value = lock.lock().unwrap();
        *value = Some(result);
        cvar.notify_one();
    }); Handle::new(handle)
  }  
}