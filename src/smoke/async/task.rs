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
use std::thread::JoinHandle;

//---------------------------------------------------------
// TaskFunc
//---------------------------------------------------------

trait TaskFunc {
    type Output;
    fn call(self: Box<Self>) -> Self::Output;
}
impl<R, F: FnOnce() -> R> TaskFunc for F {
    type Output = R;
    fn call(self: Box<Self>) -> R {
        self()
    }
}

//---------------------------------------------------------
// Task<T, E> 
//---------------------------------------------------------
pub struct Task<T, E> {
    func: Box<TaskFunc<Output = Result<T, E>> + Send + 'static>
}

impl <T, E> Task<T, E> {
    //---------------------------------------------------------
    // creates a new task.
    //---------------------------------------------------------
    #[allow(dead_code)]   
    pub fn new<F>(func: F) -> Task<T, E>
        where F: FnOnce() -> Result<T, E> + Send + 'static {
        Task {
            func: Box::new(func)
        }
    }
    //---------------------------------------------------------
    // runs this task synchronously.
    //---------------------------------------------------------
    #[allow(dead_code)]   
    pub fn sync(self) -> Result<T, E> {
        self.func.call()
    }
}

//---------------------------------------------------------
// Task<T, E> for Send + 'static
//---------------------------------------------------------
impl<T, E> Task<T, E> where T: Send + 'static,
                            E: Send + 'static {
                              
    //---------------------------------------------------------
    // executes the given tasks in parallel.
    //---------------------------------------------------------  
    #[allow(dead_code)]            
    pub fn all(tasks: Vec<Task<T, E>>) -> Task<Vec<T>, E> {
        Task::<Vec<T>, E>::new(|| {
            tasks.into_iter()
                 .map(|task| thread::spawn(|| task.sync() ) )
                 .collect::<Vec<_>>()
                 .into_iter()
                 .map(|handle| handle.join().unwrap())
                 .collect()
        })
    }
    
    //---------------------------------------------------------
    // executes this task asynchronously.
    //--------------------------------------------------------- 
    #[allow(dead_code)] 
    pub fn async(self) -> JoinHandle<Result<T, E>> {
        thread::spawn(move || self.sync())
    }
    
    //---------------------------------------------------------
    // executes this task asynchronously.
    //--------------------------------------------------------- 
    #[allow(dead_code)] 
    pub fn async_then<F, U>(self, closure: F) -> JoinHandle<U>
        where U: Send + 'static,
              F: FnOnce(Result<T, E>) -> U + Send + 'static {
        thread::spawn(move || closure(self.sync()))
    }    
}