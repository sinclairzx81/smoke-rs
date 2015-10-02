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

/// BoxedFnOnce:
///
/// Rust does not currently support Box<FnOnce>. The 
/// reason is FnOnce takes self by value, but you 
/// can't at the moment destructure a Box into the value.
/// 
/// As of writing, rust enables a feature(fnbox) to enable
/// similar functionality as below, but seems is only available
/// in nightly builds and is disallowed in stable releases.
/// The following code is effectively the same job as a the 
/// FnBox() trait until such time a stable implementation is
/// settled on.
pub trait BoxedFnOnce<T, U> {
    fn call(self: Box<Self>, arg:T) -> U;
}

impl<'a, T, U, F> BoxedFnOnce<T, U> for F 
    where F: FnOnce(T)->U + Send + 'a {
    fn call(self: Box<Self>, arg:T) -> U { 
      (*self)(arg) 
    }
}

/// Wraps a FnOnce(T)
pub struct ActionOnce<T> {
  closure: Box<BoxedFnOnce<T, ()>+Send>
}
impl<T: Send+'static> ActionOnce<T> {
  #[allow(dead_code)]
  pub fn new<F>(closure: F) -> ActionOnce<T> 
    where F: FnOnce(T) + Send + 'static {
    ActionOnce {
      closure: Box::new(closure)
    }
  }
  #[allow(dead_code)]
  pub fn call(self, value: T) {
    let _ = self.closure.call(value);
  }
}

pub struct Action<T> {
  closure: Box<Fn(T)+Send>
}
impl<T: Send+'static> Action<T> {
  #[allow(dead_code)]
  pub fn new<F>(closure: F) -> Action<T> 
    where F: Fn(T) + Send + 'static {
    Action {
      closure: Box::new(closure)
    }
  }
  #[allow(dead_code)]
  pub fn call(&self, value: T) {
    (*self.closure)(value);
  }
}