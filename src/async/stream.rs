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

use std::sync::mpsc::{SendError};
use std::sync::mpsc::{sync_channel, SyncSender};
use std::sync::mpsc::{Receiver};
use std::thread;

use super::task::Task;

/// StreamFunc<T>
///
/// Specialized FnOnce closure for Stream<T>. Provides a boxable
/// FnOnce signature for Task resolution, and acts as a fill in 
/// for a possible Box<FnOnce> capability in future. 
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

/// Stream<T> 
///
/// Provides functionality to create async sequences
/// of the given type.
pub struct Stream<T>  {
  func: Box<StreamFunc<SyncSender<T>, Output = Result<(), SendError<T>>> + Send + 'static>
}
impl<T> Stream<T> where T: Send + 'static {
  
  /// creates a new stream.
  pub fn new<F>(func:F) -> Stream<T>  where
      F: FnOnce(SyncSender<T>) -> Result<(), SendError<T>> + Send + 'static {
      Stream { func: Box::new(func) }
  }
  
  /// Begins reading from the stream. The read() function 
  /// will internally spawn a new thread for the stream
  /// sender and return to the caller a Receiver<T> in 
  /// which to recieve results. The bound given is tied
  /// to the mpsc sync_sender bound. The higher the bound,
  /// the more internally buffering. For 1 - 1 send/recv,
  /// set a value of 1.
  pub fn read(self, bound: usize) -> Receiver<T> {
      let (sync_sender, receiver) = sync_channel(bound);
      let _ = thread::spawn(move || self.func.call(sync_sender));
      receiver
  }
  
  /// The merge() function will merge multiple streams of
  /// the same type into a one stream. To merge, this
  /// function will spawn a new thread for each stream 
  /// given. The output stream will emit these with a 
  /// bound of 0 for each.
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
              Ok(send_results) => 
                  match send_results.into_iter()
                                    .collect::<Result<Vec<_>, _>>() {
                                        Err(err) => Err(err),
                                        Ok(_)    => Ok (())
                                    }
          }
      })
  }
  
  /// The filter() function will filter elements from the
  /// source stream. This function is analogous to a 
  /// filter() on a iter, with the difference being that
  /// filtering will only occur once the output stream is
  /// read.
  pub fn filter<F>(self, func:F) -> Stream<T> where 
      F: Fn(&T) -> bool + Send + 'static {
      Stream::new(move |sender|
        self.read(0).into_iter()
                    .filter(|n| func(n))
                    .map(|n| sender.send(n))
                    .last()
                    .unwrap())
  }
  
  /// The map() function will map elements from the
  /// source stream to the destination. This function 
  /// is analogous to a map() on a iter, with the 
  /// difference being that mapping  will only occur 
  /// once the output stream is read.
  pub fn map<F, U>(self, func:F) -> Stream<U> where 
    U: Send + 'static,
    F: Fn(T) -> U + Send + 'static {
      Stream::new(move |sender| 
        self.read(0).into_iter()
                    .map(|n| sender.send(func(n)))
                    .last()
                    .unwrap())
  }
  
  /// The fold() function will aggregate source elements
  /// from the stream to a single output. This function
  /// is analogous to a fold() on a iter, which the 
  /// difference being the fold will only occur once
  /// the output task is scheduled.
  pub fn fold<F>(self, init: T, func:F) -> Task<T> where 
    F: FnMut(T, T) -> T + Send + 'static {
      Task::new(move |sender|
        sender.send(self.read(0)
                        .into_iter()
                        .fold(init, func)))
  }
}
impl Stream<i32>  {
  
  /// Creates a linear sequence of i32 values from the
  /// given start and end range.
  pub fn range(start: i32, end: i32) -> Stream<i32> {
    Stream::new(move |sender| (start..end)
                  .map(|n| sender.send(n))
                  .last()
                  .unwrap())
  }
}

