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

use super::action::ActionOnce;
use std::sync::{Arc, Mutex};
use std::collections::{VecDeque};

#[allow(dead_code)]
pub struct Queue {
	queue       : Arc<Mutex<VecDeque<ActionOnce<ActionOnce<()>>>>>,
	concurrency : Arc<Mutex<u32>>,
	active      : Arc<Mutex<u32>>,
	paused      : Arc<Mutex<bool>>
}

impl Clone for Queue {
	fn clone(&self) -> Self {
		Queue {
			queue       : self.queue.clone(),
			concurrency : self.concurrency.clone(),
			active      : self.active.clone(),
			paused      : self.paused.clone()
		}
	}
}
impl Queue {
	#[allow(dead_code)]
	pub fn new(concurrency: u32) -> Queue {
		Queue {
			queue       : Arc::new(Mutex::new(VecDeque::new())),
			concurrency : Arc::new(Mutex::new(concurrency)),
			active      : Arc::new(Mutex::new(0)),
			paused      : Arc::new(Mutex::new(false))
		}
	}
	#[allow(dead_code)]
	pub fn pending(&self) -> usize {
		let queue = self.queue.lock().unwrap();
		queue.len()
	}
	
	#[allow(dead_code)]
	pub fn pause(&self) {
		let mut paused = self.paused.lock().unwrap();
		*paused = true;
	}
	
	#[allow(dead_code)]
	pub fn resume(&self) {
		{ 
			let mut paused = self.paused.lock().unwrap();
		  	*paused = false; 
		}
		self.process();
	}	
	
	#[allow(dead_code)]
	pub fn run<F: FnOnce(ActionOnce<()>)+Send+'static>(&self, operation: F) {
		{
			let mut queue = self.queue.lock().unwrap();
			queue.push_back(ActionOnce::new(operation)); 
		}
		self.process();
	}
	
	#[allow(dead_code)]
	fn increment(&self) {
		let mut active = self.active.lock().unwrap();
		*active += 1;		
	}
	
	#[allow(dead_code)]
	fn decrement(&self) {
		let mut active = self.active.lock().unwrap();
		*active -= 1;		
	}
	
	#[allow(dead_code)]
	fn process(&self) {
		let operation = {
			let mut queue   = self.queue.lock().unwrap();
			let active      = self.active.lock().unwrap();
			let concurrency = self.concurrency.lock().unwrap();
			let paused      = self.paused.lock().unwrap();
			if queue.len() > 0 && *active < *concurrency && !*paused {
				Some(queue.pop_front().unwrap())
			} else {
				None
			}
		};
		match operation {
			Some(operation) => {
				let queue_1 = self.clone();
				queue_1.increment();
				operation.call(ActionOnce::new(move |_| {
					queue_1.decrement();
					queue_1.process();					
				}));			
			}, None => {}
		}
	}
}