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

use std::sync::mpsc::{SendError};
use std::sync::mpsc::{channel, Sender};
use std::sync::mpsc::{sync_channel, SyncSender};
use std::sync::mpsc::{Receiver};
use std::thread;

use super::task::Task;

//-------------------------------------------
// StreamFunc<T> 
//-------------------------------------------

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

//-------------------------------------------
// StreamOption<T> 
//-------------------------------------------
enum StreamOption<T> {
  Sync  (SyncSender<T>),
  Async (Sender<T>)
}

//-------------------------------------------
// StreamSender<T> 
//-------------------------------------------
pub struct StreamSender<T> {
  option: StreamOption<T>
}

//-------------------------------------------
// StreamSender<T> for Send + 'static
//-------------------------------------------
impl<T> StreamSender<T> where T: Send + 'static {
  fn async(sender: Sender<T>) -> StreamSender<T> {
    StreamSender {
      option: StreamOption::Async(sender) 
    }
  }
  fn sync(sender: SyncSender<T>) -> StreamSender<T> {
    StreamSender { 
      option: StreamOption::Sync(sender) 
    }
  }

  pub fn send(&self, value:T) -> Result<(), SendError<T>> {
    match self.option {
      StreamOption::Async(ref sender) => sender.send(value),
      StreamOption::Sync (ref sender) => sender.send(value)
    }
  }
}
impl<T> Clone for StreamSender<T> where T: Send + 'static {
  fn clone(&self) -> StreamSender<T> {
    match self.option {
      StreamOption::Async(ref sender) => StreamSender::async(sender.clone()),
      StreamOption::Sync (ref sender) => StreamSender::sync(sender.clone())
    }
  }
}

//-------------------------------------------
// Stream<T> 
//-------------------------------------------
pub struct Stream<T>  {
  func: Box<StreamFunc<StreamSender<T>, Output = Result<(), SendError<T>>> + Send + 'static>
}

//-------------------------------------------
// Stream<T> for Send + 'static
//-------------------------------------------
impl<T> Stream<T> where T: Send + 'static {
    
  //-------------------------------------------
  // new(): creates a new stream.
  //-------------------------------------------  
  pub fn new<F>(func:F) -> Stream<T>  where
      F: FnOnce(StreamSender<T>) -> Result<(), SendError<T>> + Send + 'static {
      Stream {
          func: Box::new(func)
      }
  }
  
  //-------------------------------------------
  // combine(): combines N streams into 1.
  //-------------------------------------------  
  pub fn combine(streams: Vec<Stream<T>>) -> Stream<T> {
      Stream::new(move |sender| {
        let handles = streams.into_iter()
                .map(move |stream| {
                  let sender = sender.clone();
                  thread::spawn(move ||
                      stream.sync(0).into_iter()
                                    .map(|n| sender.send(n))
                                    .last()
                                    .unwrap())
                      }).collect::<Vec<_>>()
                        .into_iter()
                        .map(|handle| handle.join())
                        .collect::<Result<Vec<_>, _>>();
          match handles {
              Err(error)       => panic!(error), // thread panic.
              Ok(send_results) => match send_results.into_iter()
                                        .collect::<Result<Vec<_>, _>>() {
                                            Err(err) => Err(err),
                                            Ok(_)    => Ok (())
                                        }
          }
      })
  }
}

//---------------------------------------------------------
// Stream<T> for Send + 'static
//--------------------------------------------------------- 
impl<T> Stream<T> where T: Send + 'static {
    
  //---------------------------------------------------------
  // filter() filters elements from this stream.
  //--------------------------------------------------------- 
  pub fn filter<F>(self, func:F) -> Stream<T> where 
      F: Fn(&T) -> bool + Send + 'static {
      Stream::new(move |sender|
        self.sync(0).into_iter()
                    .filter(|n| func(n))
                    .map(|n| sender.send(n))
                    .last()
                    .unwrap())
  }
    
  //---------------------------------------------------------
  // map() maps this stream into another stream.
  //--------------------------------------------------------- 
  pub fn map<F, U>(self, func:F) -> Stream<U> where 
    U: Send + 'static,
    F: Fn(T) -> U + Send + 'static {
      Stream::new(move |sender| 
        self.sync(0).into_iter()
                    .map(|n| sender.send(func(n)))
                    .last()
                    .unwrap())
  }

  //---------------------------------------------------------
  // fold() reduces this stream to a single value.
  //---------------------------------------------------------  
  pub fn fold<F>(self, init: T, func:F) -> Task<T> where 
    F: FnMut(T, T) -> T + Send + 'static {
      Task::new(move |sender|
        sender.send(self.sync(0)
                        .into_iter()
                        .fold(init, func)))
    }
    
    //---------------------------------------------------------
    // sync() begins reading from this stream with a bound.
    //--------------------------------------------------------- 
    pub fn sync(self, bound: usize) -> Receiver<T> {
        let (sender, receiver) = sync_channel(bound);
        let _ = thread::spawn(move || 
          self.func.call(StreamSender::sync(sender)));
        receiver
    }
    
    //---------------------------------------------------------
    // async() begins reading from this stream.
    //--------------------------------------------------------- 
    pub fn async(self) -> Receiver<T> {
        let (sender, receiver) = channel();
        let _ = thread::spawn(move || 
            self.func.call(StreamSender::async(sender)));
        receiver
    }
}

//---------------------------------------------------------
// Stream<i32>
//--------------------------------------------------------- 
impl Stream<i32>  {
  
  //---------------------------------------------------------
  // range() creates a stream range.
  //--------------------------------------------------------- 
  pub fn range(start: i32, end: i32) -> Stream<i32> {
    Stream::new(move |sender| (start..end)
                  .map(|n| sender.send(n))
                  .last()
                  .unwrap())
  }
}
