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
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct Interval { 
  callback : Arc<Mutex<Box<Fn(Interval) + Send>>>,
  started  : Arc<Mutex<bool>>,
  interval : Arc<Mutex<u32>>
}
impl Interval {
  #[allow(dead_code)]
  pub fn new<F>(callback: F, interval: u32) -> Interval 
    where F: Fn(Interval) + Send + 'static {
    let this = Interval {
        callback : Arc::new(Mutex::new(Box::new(callback))),
        interval : Arc::new(Mutex::new(interval)),
        started  : Arc::new(Mutex::new(true)), 
    };
    let callback = this.callback.clone();
    let started  = this.started.clone();
    let interval = this.interval.clone();
    let clone    = this.clone();
    thread::spawn(move || {
      loop {
        let callback = callback.lock().unwrap();
        let interval = interval.lock().unwrap();
        let started  = started.lock().unwrap();
        callback(clone.clone());
        thread::sleep_ms(*interval);
        if !*started { break; } 
      }
    });
    this
  }
  
  #[allow(dead_code)]
  pub fn clear(&self) {
    let mut started = self.started.lock().unwrap();
    *started = false;      
  }
}