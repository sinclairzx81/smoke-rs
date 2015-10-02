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

use std::sync::Arc;

#[derive(Clone)]
pub struct Observer<T> {
  onnext : Arc<Box<Fn(T)>>,
  onerror: Arc<Box<Fn()>>,
  onend  : Arc<Box<Fn()>>
}

impl<T> Observer<T>
  where T: Send+'static {
  #[allow(dead_code)]
  pub fn new<Next, Error, End>
      (onnext  : Next , 
       onerror : Error,
       onend   : End) -> Observer<T> 
    where Next :  Fn(T) + Send + 'static,
          Error:  Fn()  + Send + 'static,
          End  :  Fn()  + Send + 'static {
    Observer {
      onnext  : Arc::new(Box::new(onnext)),
      onerror : Arc::new(Box::new(onerror)),
      onend   : Arc::new(Box::new(onend))
    }
  }
}

pub struct Subject<T> {
   observers: Vec<Observer<T>>
}
impl<T> Subject<T> 
  where T: Send+Clone+'static {
  
  #[allow(dead_code)]
  pub fn new() -> Subject<T> {
    Subject {
      observers: vec![]
    }
  }
  
  #[allow(dead_code)]
  pub fn onnext(&self, value:T) {
    for observer in self.observers.iter() {
      (*observer.onnext)(value.clone());
    }
  }
  
  #[allow(dead_code)]
  pub fn onerror(&self) {
    for observer in self.observers.iter() {
      (*observer.onerror)();
    }    
  }
  
  #[allow(dead_code)]
  pub fn onend(&self) {
    for observer in self.observers.iter() {
      (*observer.onend)();
    }  
  }
  
  #[allow(dead_code)]
  pub fn subscribe<Next, Error, End>
      (&mut self, onnext  : Next , 
                  onerror : Error,
                  onend   : End) -> Observer<T> 
      where Next :  Fn(T) + Send + 'static,
            Error:  Fn()  + Send + 'static,
            End  :  Fn()  + Send + 'static {
          let observer = Observer::new(onnext, onerror, onend);
          self.observers.push(observer.clone());
          observer
    }
}