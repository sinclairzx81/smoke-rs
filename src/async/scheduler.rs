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
use threadpool::ThreadPool;

//-------------------------------------------
// JoinHandle<T> 
//-------------------------------------------

pub struct JoinHandle;
impl JoinHandle {
  pub fn join(self) {
    // todo: join on the underlying thread.
  }
}

//-------------------------------------------
// ThreadScheduler<T> 
//-------------------------------------------
struct ThreadScheduler;
impl ThreadScheduler {
  fn spawn<F>(&self, f: F) -> JoinHandle where F: FnOnce() + Send + 'static {
    thread::spawn(|| {
      f();
    });
    JoinHandle
  }
}

//-------------------------------------------
// ThreadPoolScheduler<T> 
//-------------------------------------------
struct ThreadPoolScheduler {
  threadpool: ThreadPool
}
impl ThreadPoolScheduler {
  fn new(bound:usize) -> ThreadPoolScheduler {
    ThreadPoolScheduler {
      threadpool: ThreadPool::new(bound)
    }
  }
  fn spawn<F>(&self, f: F) -> JoinHandle 
    where F: FnOnce() + Send + 'static {
    self.threadpool.execute(|| {
      f();
    });
    JoinHandle
  }
}
//-------------------------------------------
// SchedulerOption<T> 
//-------------------------------------------
enum SchedulerOption {
  Thread(ThreadScheduler),
  ThreadPool(ThreadPoolScheduler)
}

//-------------------------------------------
// Scheduler<T> 
//-------------------------------------------
pub struct Scheduler {
  option: SchedulerOption
}
impl Scheduler {
  pub fn new() -> Scheduler {
    Scheduler {
      option: SchedulerOption::Thread(ThreadScheduler)
    }
  }
  pub fn pool(bound: usize) -> Scheduler {
    Scheduler {
      option: SchedulerOption::ThreadPool(ThreadPoolScheduler::new(bound))
    }
  }
  pub fn spawn<F>(&self, f: F) -> JoinHandle where 
    F: FnOnce() + Send + 'static {
    match self.option {
      SchedulerOption::Thread(ref scheduler) 
        => scheduler.spawn(f),
      SchedulerOption::ThreadPool(ref scheduler) 
        => scheduler.spawn(f)
    }
  }
}
