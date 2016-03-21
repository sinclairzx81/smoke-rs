/*--------------------------------------------------------------------------
 smoke-rs

 The MIT License (MIT)

 Copyright (c) 2016 Haydn Paterson (sinclair) <haydn.developer@gmail.com>

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

use std::sync::mpsc::{Receiver, RecvError};

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
pub struct Handle<T> {
  receiver: Receiver<T>
}
impl<T> Handle<T> where T: Send + 'static {
  
  /// Creates a new wait handle. Wait handles are created
  /// by schedulers when running tasks. When the task is
  /// being run, a sync_channel is created, the sending
  /// end is passed to the task, the receiving end is passed
  /// here.
  pub fn new(receiver: Receiver<T>) -> Handle<T> {
    Handle { receiver: receiver }
  }
  
  /// Waits on the handles receiver. This method
  /// will block the current thread while waiting
  /// for a result.
  pub fn wait(self) -> Result<T, RecvError> {
    self.receiver.recv()
  }
}