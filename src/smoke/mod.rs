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


pub mod async;
pub mod threads;
pub mod process;
pub mod stream;
pub mod file;
pub mod tcp;
pub mod timers;


use self::async::Task;

pub trait ReadAsync {
	fn pause  (&self) -> ();
	fn resume (&self) -> ();
	fn ondata  <F>(&self, action: F) -> Self where F: Fn((Self, Vec<u8>)) + Send + 'static;
	fn onerror <F>(&self, action: F) -> Self where F: FnOnce((Self, Error)) + Send + 'static;
	fn onend   <F>(&self, action: F) -> Self where F: FnOnce((Self, )) + Send + 'static;
}

pub trait WriteAsync {
	fn write (&self, data: Vec<u8>) -> Task<(), Error>;
	fn end   (&self)                -> Task<(), Error>;
}

#[derive(Clone)]
pub struct Error {
	message: String
}
impl Error {
	pub fn new(message: &'static str) -> Error {
		Error {
			message: message.to_string()
		}
	}
}
use std::fmt::{Display, Debug, Formatter, Result};
impl Display for Error {
	fn fmt(&self, f: &mut Formatter) -> Result {
			write!(f, "{}", self.message)
	}
}
impl Debug for Error {
	fn fmt(&self, f: &mut Formatter) -> Result {
		write!(f, "{}", self.message)
	}
}