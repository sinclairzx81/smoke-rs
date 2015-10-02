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

use super::super::async::{Task, Queue};
use std::sync::{Arc, Mutex};
use std::io::{Write, Error};
use std::thread;

#[derive(Clone)]
pub struct Writer {
	stream : Arc<Mutex<Box<Write+Send>>>,
	queue  : Queue
}

impl Writer {
	pub fn new<F>(stream: F) -> Writer
		where F: Write+Send+'static {
		Writer {
			stream : Arc::new(Mutex::new(Box::new(stream))),
			queue  : Queue::new(1)
		}
	}
	
	#[allow(dead_code)]
	pub fn write(&self, data: Vec<u8>) -> Task<(), Error> {
    let this = self.clone();
    Task::new(move |task| {
      let queue = this.queue.clone();
      queue.run(move |next| {
        thread::spawn(move || {
          let mut stream = this.stream.lock().unwrap();			
					stream.write_all(&data).unwrap();
          task.call(Ok(()));
          next.call(());
        });
      })
    })		
	}
	
	#[allow(dead_code)]
	pub fn end(self) -> Task<(), Error> {
    let this = self.clone();
    Task::new(move |task| {
      let queue = this.queue.clone();
      queue.run(move |next| {
        	task.call(Ok(()));
        	next.call(());
      })
    })
	}
}