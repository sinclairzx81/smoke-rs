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
use std::sync::mpsc::channel;
use std::sync::mpsc::{Sender, Receiver, SendError};

//---------------------------------------------------------
// SenderFunc<T> -> TResult 
//---------------------------------------------------------
trait SenderFunc<T> {
    type Output;
    fn call(self: Box<Self>, arg: T) -> Self::Output;
}
impl<T, TResult, F: FnOnce(T) -> TResult> SenderFunc<T> for F {
    type Output = TResult;
    fn call(self: Box<Self>, value: T) -> TResult {
        self(value)
    }
}

//---------------------------------------------------------
// Stream<T>
//---------------------------------------------------------
pub struct Stream<T> {
    func: Box <SenderFunc<Sender<T>, Output = Result<(), SendError<T>>> + Send + 'static>
}
impl <T> Stream<T> {
    
    //---------------------------------------------------------
    // creates a new Stream<T>
    //---------------------------------------------------------
    #[allow(dead_code)]    
    pub fn new<F>(func:F) -> Stream<T>
        where F: FnOnce(Sender<T>) -> Result<(), SendError<T>> + Send + 'static {
        Stream {
            func: Box::new(func)   
        }
    }
}
//------------------------------
// Stream<T> for Send + 'static
//------------------------------
impl <T> Stream<T> where T: Send + 'static {
    
    //---------------------------------------------------------
    // maps stream to another stream.
    //---------------------------------------------------------
    #[allow(dead_code)]   
    pub fn map<F, U>(self, func:F) -> Stream<U>
        where F: Fn(T) -> U + Send + 'static {
        Stream::new(move |sender| {
            for value in self.recv() {
                try!(sender.send(func(value)));
            } Ok(())
        })
    }
    
    //---------------------------------------------------------
    // filters stream
    //---------------------------------------------------------
    #[allow(dead_code)]   
    pub fn filter<F>(self, func:F) -> Stream<T>
        where F: Fn(&T) -> bool + Send + 'static {
        Stream::new(move |sender| {
            for value in self.recv() {
                if func(&value) {
                    try!(sender.send(value));
                }
            } Ok(())
        })
    }
    
    //---------------------------------------------------------
    // merges streams into a single stream.
    //---------------------------------------------------------
    #[allow(dead_code)]   
    pub fn merge(streams: Vec<Stream<T>>) -> Stream<T> {
        Stream::new(move |sender| {
            // Stream<T> -> JoinHandle<T>
            let handles = streams.into_iter().map(move |stream| {
                let sender = sender.clone();
                thread::spawn::<_, Result<(), SendError<T>>>(move || {
                    for value in stream.recv() {
                        try!(sender.send(value));
                    } Ok(())
                })
            }).collect::<Vec<_>>()
              .into_iter()
              .map(|handle| handle.join())
              .collect::<Result<Vec<_>, _>>();
              
             // process handles
             match handles {
                 Err(_) => panic!("thread paniced"),
                 Ok(send_results) => {
                      // process send results..
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
    
    //---------------------------------------------------------
    // syncs this stream, returns a Receiver<T>
    //---------------------------------------------------------
    #[allow(dead_code)]   
    pub fn recv(self) -> Receiver<T> {
        let (sender, receiver) = channel();
        thread::spawn(move || self.func.call(sender)); 
        receiver
    }  
}