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

extern crate threadpool;

use std::sync::mpsc::{sync_channel, Receiver, RecvError};
use super::task::{Task, TaskSender};

/// WaitHandle<T>
///
/// A waitable handle for scheduled tasks.
pub struct WaitHandle<T> {
  receiver: Receiver<T>
}
impl<T> WaitHandle<T> where T: Send + 'static {
  
  /// Creates a new wait handle. Wait handles are created
  /// by schedulers when running tasks. When the task is
  /// being run, a sync_channel is created, the sending
  /// end is passed to the task, the receiving end is passed
  /// here.
  fn new(receiver: Receiver<T>) -> WaitHandle<T> {
    WaitHandle { receiver: receiver }
  }
  
  /// Waits on the handles receiver. This method
  /// will block the current thread while waiting
  /// for a result.
  pub fn wait(self) -> Result<T, RecvError> {
    self.receiver.recv()
  }
}


/// Scheduler
///
/// Common scheduler trait implemented by all schedulers.
pub trait Scheduler {
  
  /// Will schedule a task to be run. Returns a WaitHandle<T> to the caller
  /// which may optionally be waited on. 
  fn run<T>(&self, task: Task<T>) -> WaitHandle<T> where T: Send + 'static;
}

/// SyncScheduler
///
/// A synchronous scheduler. Tasks scheduled on this scheduler
/// will be executed on the current thread, potentially blocking
/// other operations.
pub struct SyncScheduler;
impl SyncScheduler {
  
  /// Creates a new SyncScheduler.
  pub fn new() -> SyncScheduler {
    SyncScheduler
  }
}
impl Scheduler for SyncScheduler {
  fn run<T>(&self, task: Task<T>) -> WaitHandle<T> where T: Send + 'static {
    let (sender, receiver) = sync_channel(1);
    let handle = WaitHandle::new(receiver);
    match task.func.call(TaskSender::new(sender)) {
      Err(error) => panic!(format!("Scheduler: Error processing task: {}", error)),
      Ok (_)     => { /* ... */ }
    };
    handle
  }
}

/// ThreadScheduler
///
/// A asynchronous scheduler. Tasks scheduled here are executed
/// on their own dedicated threads. When using this scheduler, all
/// threads are unbounded. For bounded threads, consider using
/// the ThreadPoolScheduler.
use std::thread;
pub struct ThreadScheduler;
impl ThreadScheduler {
  /// Creates a new ThreadScheduler.
  pub fn new() -> ThreadScheduler {
    ThreadScheduler
  }
}
impl Scheduler for ThreadScheduler {
  fn run<T>(&self, task: Task<T>) -> WaitHandle<T> where T: Send + 'static {
    let (sender, receiver) = sync_channel(1);
    let handle = WaitHandle::new(receiver);
    thread::spawn(move || {
      match task.func.call(TaskSender::new(sender)) {
        Err(error) => panic!(format!("Scheduler: Error processing task: {}", error)),
        Ok (_)     => { /* ... */ }
      }
    }); handle
  }
}
/// ThreadPoolScheduler
///
/// A asynchronous scheduler. Tasks scheduled here are executed
/// in a pool of threads specified by the caller at creation,
/// any threads scheduled here will be queued if the pool of 
/// threads is busy.
use self::threadpool::ThreadPool;
pub struct ThreadPoolScheduler {
  threadpool: ThreadPool
}
impl ThreadPoolScheduler {
  
  /// Creates a new ThreadPoolScheduler with the given number of threads.
  pub fn new(threads: usize) -> ThreadPoolScheduler {
    let threadpool = ThreadPool::new(threads);
    ThreadPoolScheduler {
      threadpool: threadpool 
    }
  }
}
impl Scheduler for ThreadPoolScheduler {
  fn run<T>(&self, task: Task<T>) -> WaitHandle<T> where T: Send + 'static {
    let (sender, receiver) = sync_channel(1);
    let handle = WaitHandle::new(receiver);
    self.threadpool.execute(move || {
      match task.func.call(TaskSender::new(sender)) {
        Err(error) => panic!(format!("Scheduler: Error processing task: {}", error)),
        Ok (_)     => { /* ... */ }
      }
    }); handle
  }
}