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

/// Specialized boxed FnOnce() closure type for streams.
pub trait StreamFunc<T> {
    type Output;
    fn call(self: Box<Self>, arg: T) -> Self::Output;
}
impl<T, TResult, F: FnOnce(T) -> TResult> StreamFunc<T> for F {
    type Output = TResult;
    fn call(self: Box<Self>, value: T) -> TResult {
        self(value)
    }
}

/// Provides functionality to generate asynchronous sequences.
pub struct Stream<T>  {
  /// The closure used to emit elements on this stream.
  pub func: Box<StreamFunc<SyncSender<T>, Output = Result<(), SendError<T>>> + Send + 'static>
}
impl<T> Stream<T> where T: Send + 'static {
  
  /// Creates a new stream.
  ///
  /// # Example
  ///
  /// ```
  /// use smoke::async::Stream;
  /// 
  /// fn numbers() -> Stream<i32> {
  ///   Stream::new(|sender| {
  ///      try!(sender.send(1));
  ///      try!(sender.send(2));
  ///      try!(sender.send(3));
  ///      sender.send(4)    
  ///   })
  /// }
  /// ```
  pub fn new<F>(func:F) -> Stream<T>  where
      F: FnOnce(SyncSender<T>) -> Result<(), SendError<T>> + Send + 'static {
      Stream { func: Box::new(func) }
  }
  
  /// Reads elements from the stream.
  /// # Example
  ///
  /// ```
  /// use smoke::async::Stream;
  ///
  /// for n in Stream::range(0, 4).read(0) {
  ///     // 0, 1, 2, 3
  /// } 
  pub fn read(self, bound: usize) -> Receiver<T> {
      let (sync_sender, receiver) = sync_channel(bound);
      let _ = thread::spawn(move || self.func.call(sync_sender));
      receiver
  }
  
  /// Will merge multiple streams into a single stream. 
  /// # Example
  ///
  /// ```
  /// use smoke::async::Stream;
  ///
  /// let a = Stream::range(0, 4);
  /// let b = Stream::range(4, 8);
  /// let c = Stream::merge(vec![a, b]);
  /// for n in c.read(0) {
  ///     // 0, 1, 2, 3, 4, 5, 6, 7
  /// }
  /// ```
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
  
  /// Will filter elements from the source stream.
  /// # Example
  ///
  /// ```
  /// use smoke::async::Stream;
  ///
  /// let numbers = Stream::range(0, 100);
  /// let evens   = numbers.filter(|n| n % 2 == 0);
  /// for n in evens.read(0) {
  ///     // only even numbers
  /// }
  /// ```
  pub fn filter<F>(self, func:F) -> Stream<T> where 
      F: Fn(&T) -> bool + Send + 'static {
      Stream::new(move |sender|
        self.read(0).into_iter()
                    .filter(|n| func(n))
                    .map(|n| sender.send(n))
                    .last()
                    .unwrap())
  }
  
  /// Will map the source stream into a new stream.
  /// # Example
  ///
  /// ```
  /// use smoke::async::Stream;
  ///
  /// let numbers = Stream::range(0, 100);
  /// let strings = numbers.map(|n| format!("number {}", n));
  /// for n in strings.read(0) {
  ///     // strings
  /// }
  /// ```    
  pub fn map<F, U>(self, func:F) -> Stream<U> where 
    U: Send + 'static,
    F: Fn(T) -> U + Send + 'static {
      Stream::new(move |sender| 
        self.read(0).into_iter()
                    .map(|n| sender.send(func(n)))
                    .last()
                    .unwrap())
  }
  
  /// Reduces elements in the source stream and returns a task
  /// to obtain the result.
  /// # Example
  ///
  /// ```
  /// use smoke::async::Stream;
  ///
  /// let numbers  = Stream::range(0, 100);
  /// let task     = numbers.fold(0, |p, c| p + c);
  /// let result   = task.wait().unwrap();
  /// ```  
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
  /// # Example
  ///
  /// ```
  /// use smoke::async::Stream;
  ///
  /// let numbers = Stream::range(0, 100);
  /// for n in numbers.read(0) {
  ///     // only even numbers
  /// }
  /// ``` 
  pub fn range(start: i32, end: i32) -> Stream<i32> {
    Stream::new(move |sender| (start..end)
                  .map(|n| sender.send(n))
                  .last()
                  .unwrap())
  }
}

/// Trait implemented for types that can be converted into streams.
pub trait ToStream<T> {
  
  /// Converts this type to a stream.
  ///
  /// #Example
  /// ```
  /// use smoke::async::ToStream;
  /// 
  /// let stream = (0 .. 10).to_stream();
  /// 
  /// for n in stream.read(0) {
  ///   println!("{}", n);
  /// }
  /// ```  
  fn to_stream(self: Self) -> Stream<T>;
}

impl <F: Iterator<Item = T> + Send + 'static, T: Send + 'static> ToStream<T> for F {
  /// Converts this type to a stream.
  fn to_stream(self: Self) -> Stream<T> {
    Stream::new(|sender| {
      for n in self {
        try!( sender.send(n) );
      } Ok(())
    })
  }
}