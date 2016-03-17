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

use std::sync::mpsc::{SendError};
use std::sync::mpsc::{sync_channel, SyncSender};
use std::sync::mpsc::{Receiver};
use std::thread;

use super::task::Task;

///---------------------------------------------------------
/// TaskFunc<T>
///
/// Specialized FnOnce closure for Stream<T>. Provides a boxable
/// FnOnce signature for Task resolution, and acts as a fill in 
/// for a possible Box<FnOnce> capability in future. 
///---------------------------------------------------------
trait StreamFunc<T> {
    type Output;
    fn call(self: Box<Self>, arg: T) -> Self::Output;
}
impl<T, TResult, F: FnOnce(T) -> TResult> StreamFunc<T> for F {
    type Output = TResult;
    fn call(self: Box<Self>, value: T) -> TResult {
        self(value)
    }
}

///---------------------------------------------------------
/// Stream<T> 
///
/// Provides functionality to create async enumerable 
/// sequences. Supports async buffering by way of
/// mpsc sync_channel. A stream is given its own isolated
/// thread in which to stream.
///---------------------------------------------------------
pub struct Stream<T>  {
  func: Box<StreamFunc<SyncSender<T>, Output = Result<(), SendError<T>>> + Send + 'static>
}
impl<T> Stream<T> where T: Send + 'static {
  
  ///---------------------------------------------------------
  /// creates a new stream.
  ///---------------------------------------------------------
  pub fn new<F>(func:F) -> Stream<T>  where
      F: FnOnce(SyncSender<T>) -> Result<(), SendError<T>> + Send + 'static {
      Stream { func: Box::new(func) }
  }
  
  ///---------------------------------------------------------
  /// starts reading from the stream. The bound given
  /// relates to the sync_channel bound. For 1 to 1 send / recv
  /// use a bound value of 0.
  ///---------------------------------------------------------
  pub fn read(self, bound: usize) -> Receiver<T> {
      let (sync_sender, receiver) = sync_channel(bound);
      let _ = thread::spawn(move || self.func.call(sync_sender));
      receiver
  }
  
  ///---------------------------------------------------------
  /// merges multiple streams of the same type into a single 
  /// stream.
  ///---------------------------------------------------------
  pub fn merge(streams: Vec<Stream<T>>) -> Stream<T> {
      Stream::new(move |sender| {
        let handles = streams.into_iter()
                .map(move |stream| {
                  let sender = sender.clone();
                  thread::spawn(move ||
                      stream.read(0).into_iter()
                                    .map(|n| sender.send(n))
                                    .last()
                                    .unwrap())
                      }).collect::<Vec<_>>()
                        .into_iter()
                        .map(|handle| handle.join())
                        .collect::<Result<Vec<_>, _>>();
          match handles {
              Err(error) => panic!(error),
              Ok(send_results) => match send_results.into_iter()
                                        .collect::<Result<Vec<_>, _>>() {
                                            Err(err) => Err(err),
                                            Ok(_)    => Ok (())
                                        }
          }
      })
  }
  ///---------------------------------------------------------
  /// Creates a new stream that will filter elements from the
  /// source. 
  ///---------------------------------------------------------
  pub fn filter<F>(self, func:F) -> Stream<T> where 
      F: Fn(&T) -> bool + Send + 'static {
      Stream::new(move |sender|
        self.read(0).into_iter()
                    .filter(|n| func(n))
                    .map(|n| sender.send(n))
                    .last()
                    .unwrap())
  }
    
  ///---------------------------------------------------------
  /// Creates a new stream that maps elements from the
  /// source. 
  ///---------------------------------------------------------
  pub fn map<F, U>(self, func:F) -> Stream<U> where 
    U: Send + 'static,
    F: Fn(T) -> U + Send + 'static {
      Stream::new(move |sender| 
        self.read(0).into_iter()
                    .map(|n| sender.send(func(n)))
                    .last()
                    .unwrap())
  }
  
  ///---------------------------------------------------------
  /// Reduces this stream into a single result. The result
  /// is given on the Task.
  ///---------------------------------------------------------  
  pub fn fold<F>(self, init: T, func:F) -> Task<T> where 
    F: FnMut(T, T) -> T + Send + 'static {
      Task::new(move |sender|
        sender.send(self.read(0)
                        .into_iter()
                        .fold(init, func)))
  }
}
impl Stream<i32>  {
  
  ///---------------------------------------------------------
  /// Creates a new stream with the given range.
  ///---------------------------------------------------------  
  pub fn range(start: i32, end: i32) -> Stream<i32> {
    Stream::new(move |sender| (start..end)
                  .map(|n| sender.send(n))
                  .last()
                  .unwrap())
  }
}

