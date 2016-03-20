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
use self::threadpool::ThreadPool;
use std::thread;


/// A waitable handle for scheduled issused by schedulers running tasks.
///
/// # Examples
/// ```
/// use smoke::async::Task;
/// use smoke::async::{Scheduler, ThreadScheduler};
/// fn hello() -> Task<&'static str> {
///   Task::delay(1).map(|_| "hello")
/// }
///
/// fn main() {
///   let scheduler = ThreadScheduler;
///   let handle    = scheduler.run(hello());
///   // sometime later...
///   println!("{:?}", handle.wait());
/// }
/// ```
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


/// Common scheduler trait implemented by all schedulers.
pub trait Scheduler {
  
  /// Schedules a task.
  fn run<T>(&self, task: Task<T>) -> WaitHandle<T> where T: Send + 'static;
}

/// A synchronous scheduler. Tasks scheduled on this scheduler
/// will be executed on the current thread, potentially blocking
/// other operations.
///
/// # Examples
/// ```
/// use smoke::async::Task;
/// use smoke::async::SyncScheduler;
///
/// fn hello() -> Task<&'static str> {
///   Task::delay(1).map(|_| "hello")
/// }
///
/// fn main() {
///   let scheduler = SyncScheduler;
///   let handle    = hello().schedule(scheduler);
///   // sometime later...
///   println!("{:?}", handle.wait());
/// }
/// ```
pub struct SyncScheduler;
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

/// A asynchronous scheduler. Tasks scheduled here are executed
/// on their own dedicated threads. When using this scheduler, all
/// threads are unbounded. For bounded threads, consider using
/// the ThreadPoolScheduler.
///
/// # Examples
/// ```
/// use smoke::async::Task;
/// use smoke::async::ThreadScheduler;
///
/// fn hello() -> Task<&'static str> {
///   Task::delay(1).map(|_| "hello")
/// }
///
/// fn main() {
///   let scheduler = ThreadScheduler;
///   let handle    = hello().schedule(scheduler);
///   // sometime later...
///   println!("{:?}", handle.wait());
/// }
/// ```
pub struct ThreadScheduler;
impl ThreadScheduler {
  /// Creates a new thread scheduler.
  pub fn new() -> ThreadScheduler {
    ThreadScheduler
  }
}
impl Scheduler for ThreadScheduler {
  /// Schedules a task.
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

/// A asynchronous scheduler. Tasks scheduled here are executed
/// within a threadpool of the given size.
///
/// # Examples
/// ```
/// use smoke::async::Task;
/// use smoke::async::ThreadPoolScheduler;
///
/// fn hello() -> Task<&'static str> {
///   Task::delay(1).map(|_| "hello")
/// }
///
/// fn main() {
///   let scheduler = ThreadPoolScheduler::new(8);
///   let handle    = hello().schedule(scheduler);
///   // sometime later...
///   println!("{:?}", handle.wait());
/// }
/// ```
pub struct ThreadPoolScheduler {
  threadpool: ThreadPool
}
impl ThreadPoolScheduler {
  
  /// Creates a new threadpool scheduler with the given number of threads.
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