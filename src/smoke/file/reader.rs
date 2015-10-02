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

pub use super::super::{Error, ReadAsync, WriteAsync};
use super::super::async::{Event};
use super::super::stream::Reader;
use std::sync::{Arc, Mutex};
use std::io::prelude::*;
use std::fs::File;

#[derive(Clone)]
pub struct FileReader {
	reader    : Reader,
	onread    : Event<(FileReader, Vec<u8>)>,
	onerror   : Event<(FileReader, Error)>,
	onend     : Event<(FileReader, )>,
	reading   : Arc<Mutex<bool>>
}

impl FileReader {
	#[allow(dead_code)]
	pub fn new(path: &'static str) -> FileReader {
		FileReader {
			reader   : Reader::new(File::open(path).unwrap(), 165536),
			onread   : Event::new(),
			onerror  : Event::new(),
			onend    : Event::new(),
			reading  : Arc::new(Mutex::new(false))			
		}
	}
	
	/// reads from the stream while reading is true.
	fn read(&self) {
		let this = self.clone();
		self.reader.read().then(move |result| {
				match result {
					Ok(data) => {
						if data.len() > 0 {
							this.onread.emit((this.clone(), data));
							let reading = {
								*this.reading.lock().unwrap()
							};
							if reading {
								this.read();
							}
						} else {
							this.onend.emit((this.clone(), ));
						}
					},
					Err(_) => this.onerror.emit((this.clone(), Error::new("error reading from the socket")))
				}
			}).async();
	}	
}

impl ReadAsync for FileReader {	
	
	fn ondata<F>(&self, action: F) -> FileReader
		where F: Fn((FileReader, Vec<u8>))+Send+'static  {
		self.onread.on(action);
		self.resume();
		self.clone()
	}
	
	fn onerror<F>(&self, action: F) -> FileReader
		where F: FnOnce((FileReader, Error))+Send+'static {
		self.onerror.once(action);
		self.clone()
	}
	
	fn onend<F>(&self, action: F) -> FileReader
		where F: FnOnce((FileReader, ))+Send+'static {
		self.onend.once(action);
		self.resume();
		self.clone()
	}
	
	fn pause(&self) {
		let mut reading = self.reading.lock().unwrap();
		*reading = false;
	}
	
	fn resume(&self) {
		let should_resume = {
			let mut reading = self.reading.lock().unwrap();
			if !*reading { 
				*reading = true;
				true
			} else { false }
		}; if should_resume { self.read(); }
	}
}