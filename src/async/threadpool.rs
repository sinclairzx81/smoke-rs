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
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

//---------------------------------------------------------
// ThreadPoolFunc
//---------------------------------------------------------

trait ThreadPoolFunc {
    type Output;
    fn call(self: Box<Self>) -> Self::Output;
}
impl< TResult, F: FnOnce() -> TResult> ThreadPoolFunc for F {
    type Output = TResult;
    fn call(self: Box<Self>) -> TResult {
        self()
    }
}

//---------------------------------------------------------
// ThreadPoolData
//---------------------------------------------------------
struct ThreadPoolData {
    queue  : VecDeque<Box<ThreadPoolFunc<Output=()> + Send + 'static>>,
    bound  : usize,
    active : usize
}
//---------------------------------------------------------
// ThreadPool
//---------------------------------------------------------
#[derive(Clone)]
pub struct ThreadPool {
	data: Arc<Mutex<ThreadPoolData>>
}
impl ThreadPool {
    
    //---------------------------------------------------------
    // new() creates a new threadpool.
    //---------------------------------------------------------
    pub fn new(bound: usize) -> ThreadPool {
        ThreadPool {
            data: Arc::new(Mutex::new(ThreadPoolData {
                 queue : VecDeque::new(),
                 bound : bound,
                 active: 0
            }))
        }
    }
    //---------------------------------------------------------
    // increment() increments the active count.
    //---------------------------------------------------------    
    fn increment(&self) {
        let mut data = self.data.lock().unwrap();
        data.active += 1;		
    }
    //---------------------------------------------------------
    // decrement() decrements the active count.
    //---------------------------------------------------------    
    fn decrement(&self) {
        let mut data = self.data.lock().unwrap();
        data.active -= 1;		
    }
    //---------------------------------------------------------
    // process() creates a new threadpool.
    //---------------------------------------------------------
    fn process(&self) {
        let result = {
            let mut data = self.data.lock().unwrap();
            if data.queue.len() > 0 && data.active < data.bound {
                Some(data.queue.pop_front().unwrap())
            } else {
                None
            }
        };
        match result {
            Some(func) => {
                self.increment();
                let pool = self.clone();
                thread::spawn(move || {
                    func.call();
                    pool.decrement();
                    pool.process();
                });	
            }, None => { /* do nothing */ }
        }
    }
    
    //---------------------------------------------------------
    // spawn() spawns a new thread.
    //---------------------------------------------------------    
    pub fn spawn<F>(&self, func: F) where 
        F: FnOnce() -> () + Send + 'static {
        {
            let mut data = self.data.lock().unwrap();
            data.queue.push_back(Box::new(func)); 
        } self.process();
    }
}