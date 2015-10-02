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

use super::action::Action;
use super::action::ActionOnce;
use std::sync::{Arc, Mutex};
use std::mem;

#[derive(Clone)]
pub struct Event<T> {
	ons  : Arc<Mutex<Vec<Action<T>>>>,
	onces: Arc<Mutex<Vec<ActionOnce<T>>>>
}

impl<T> Event<T> where T: Clone+Send+'static {
	
	#[allow(dead_code)]
	pub fn new() -> Event<T> {
		Event {
			ons   : Arc::new(Mutex::new(Vec::new())),
			onces : Arc::new(Mutex::new(Vec::new()))
		}
	}
	
	#[allow(dead_code)]
	pub fn on<F>(&self, closure: F) 
	where F: Fn(T)+Send+'static {
		let mut ons = self.ons.lock().unwrap();
		ons.push(Action::new(closure))
	}
		
	#[allow(dead_code)]
	pub fn once<F>(&self, closure: F)
	   where F: FnOnce(T)+Send+'static {
		let mut onces = self.onces.lock().unwrap();
		onces.push(ActionOnce::new(closure))		  
	}
	
	#[allow(dead_code)]
	pub fn emit(&self, value: T) {
		let ons = self.ons.lock().unwrap();
		for on in ons.iter() {
			on.call(value.clone());
		} 
		
		let mut onces = self.onces.lock().unwrap();
		let onces = mem::replace(&mut *onces, Vec::new());
		for once in onces {
			once.call(value.clone());
		}	
	} 
}