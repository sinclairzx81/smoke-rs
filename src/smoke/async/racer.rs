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

use std::sync::{Arc, Mutex};

pub struct Racer {
	complete: Arc<Mutex<bool>>
}
impl Clone for Racer {
	fn clone(&self) -> Self {
		Racer { 
			complete: self.complete.clone()
		}
	}
}
impl Racer {
	#[allow(dead_code)]
	pub fn new() -> Racer {
		Racer { complete: Arc::new(Mutex::new(false)) }
	}
	#[allow(dead_code)]
	pub fn set<F: FnOnce()+Send+'static>(&self, action: F)  {
		let mut complete = self.complete.lock().unwrap();
		if *complete { return }
		*complete = true;
		action();
	}
}