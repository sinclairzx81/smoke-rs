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
use std::sync::mpsc::{SendError};
use std::sync::mpsc::{channel, Sender};
use std::sync::mpsc::{sync_channel, SyncSender};
use std::sync::mpsc::{Receiver};

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
// StreamSender<T> 
//-------------------------------------------
enum SenderOption<T> {
  Sync  (SyncSender<T>),
  Async (Sender<T>)
}
pub struct StreamSender<T> {
  option: SenderOption<T>
}
impl<T> StreamSender<T> where T: Send + 'static {
  fn async(sender: Sender<T>) -> StreamSender<T> {
    StreamSender { option: SenderOption::Async(sender) }
  }
  fn sync(sender: SyncSender<T>) -> StreamSender<T> {
    StreamSender { option: SenderOption::Sync(sender) }
  }
  pub fn send(&self, value:T) -> Result<(), SendError<T>> {
    match self.option {
      SenderOption::Async(ref sender) 
        => sender.send(value),
      SenderOption::Sync (ref sender)  
        => sender.send(value)
    }
  }
}
impl<T> Clone for StreamSender<T> {
  fn clone(&self) -> StreamSender<T> {
    match self.option {
      SenderOption::Async(ref sender) 
        => StreamSender { option: SenderOption::Async(sender.clone()) },
      SenderOption::Sync (ref sender) 
        => StreamSender { option: SenderOption::Sync (sender.clone()) }
    }
  }
}
//-------------------------------------------
// Stream<T> 
//-------------------------------------------
pub struct Stream<T>  {
  func: Box<StreamFunc<StreamSender<T>, Output = Result<(), SendError<T>>> + Send + 'static>
}
impl<T> Stream<T> where T: Send + 'static {
    pub fn new<F>(func:F) -> Stream<T>
        where F: FnOnce(StreamSender<T>) -> Result<(), SendError<T>> + Send + 'static {
        Stream {
            func: Box::new(func)
        }
    }
    pub fn all(streams: Vec<Stream<T>>) -> Stream<T> {
        Stream::new(move |sender| {
            let handles = streams.into_iter().map(move |stream| {
                let sender = sender.clone();
                thread::spawn::<_, Result<(), SendError<T>>>(move || {
                    for value in stream.sync(0) {
                        try!(sender.send(value));
                    } Ok(())
                })
            }).collect::<Vec<_>>()
              .into_iter()
              .map(|handle| handle.join())
              .collect::<Result<Vec<_>, _>>();
             match handles {
                 Err(_) => panic!("thread paniced"),
                 Ok(send_results) => {
                     let results = 
                        send_results
                            .into_iter()
                            .collect::<Result<Vec<_>, _>>();
                    match results {
                        Err(err) => Err(err),
                        Ok(_) => Ok(())
                    }
                 }
             }
        })
    }
}
impl<T> Stream<T> where T: Send + 'static {
    
    pub fn filter<F>(self, func:F) -> Stream<T>
        where F: Fn(&T) -> bool + Send + 'static {
        Stream::new(move |sender| {
            for value in self.sync(0) {
                if func(&value) {
                    try!(sender.send(value));
                }
            } Ok(())
        })
    }
    
    pub fn map<F, U>(self, func:F) -> Stream<U>
        where U: Send + 'static,
              F: Fn(T) -> U + Send + 'static {
        Stream::new(move |sender| {
            for value in self.sync(0) {
                try!(sender.send(func(value)));
            } Ok(())
        })
    }
    
    pub fn reduce<F>(self, func:F, value: T) -> Task<T, ()>
        where F: Fn(T, T) -> T + Send + 'static {
        Task::new(move || {
            let mut accumulator = value;
            for value in self.sync(0) {
                accumulator = func(accumulator, value)
            } Ok(accumulator)
        })
    }
    
    pub fn sync(self, bound: usize) -> Receiver<T> {
        let (sender, receiver) = sync_channel(bound);
        thread::spawn(move || {
          let emit = StreamSender::sync(sender);
          self.func.call(emit).unwrap();
          });
        receiver
    }
    
    pub fn async(self) -> Receiver<T> {
        let (sender, receiver) = channel();
        thread::spawn(move || {
            let emit = StreamSender::async(sender);
            self.func.call(emit).unwrap();
          });
        receiver
    }
}