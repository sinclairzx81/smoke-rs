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
use std::io::{Read, Error};
use std::thread;

#[derive(Clone)]
pub struct Reader {
  stream : Arc<Mutex<Box<Read+Send>>>,
	buffer : Arc<Mutex<Vec<u8>>>,
	queue  : Queue
}
impl Reader {
	#[allow(dead_code)]
	pub fn new<F>(read: F, buffersize: usize) -> Reader 
    where F: Send+Read+'static {
		Reader {
      stream : Arc::new(Mutex::new(Box::new(read))),
			buffer : Arc::new(Mutex::new(vec![0; buffersize])),
			queue  : Queue::new(1)
		}
	}
  pub fn read(&self) -> Task<Vec<u8>, Error> {
    let this = self.clone();
    Task::new(move |task| {
      let queue = this.queue.clone();
      queue.run(move |next| {
        thread::spawn(move || {
          task.call(this.next());
          next.call(());
        });
      })
    })
  }
  pub fn next(&self) -> Result<Vec<u8>, Error> {
      let mut stream = self.stream.lock().unwrap();
      let mut buffer = self.buffer.lock().unwrap();
      let read       = stream.read(&mut *buffer).unwrap();        
      let buffer     = &buffer[0..read];
      let buffer     = buffer.to_vec();
      Ok(buffer) 
  }
}