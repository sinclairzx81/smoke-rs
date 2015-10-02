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
use super::super::async::{Task, Event, Queue};
use super::super::stream::{Reader, Writer};
use std::net::{TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Clone)]
pub struct Socket {
	reader    : Arc<Mutex<Option<Reader>>>,
	writer    : Arc<Mutex<Option<Writer>>>,
	queue     : Queue,
	onconnect : Event<(Socket, )>,
	onread    : Event<(Socket, Vec<u8>)>,
	onerror   : Event<(Socket, Error)>,
	onend     : Event<(Socket, )>,
	reading   : Arc<Mutex<bool>>
}

impl Socket {
	/// creates a socket from a stream.
	#[allow(dead_code)]
	pub fn from_stream(stream: TcpStream) -> Socket {
		let stream_1 = stream.try_clone().unwrap();
		let stream_2 = stream.try_clone().unwrap();
		Socket {
			reader   : Arc::new(Mutex::new(Some(Reader::new(stream_1, 4096)))),
			writer   : Arc::new(Mutex::new(Some(Writer::new(stream_2)))),
			queue    : Queue::new(1),
			onconnect: Event::new(),
			onread   : Event::new(),
			onerror  : Event::new(),
			onend    : Event::new(),
			reading  : Arc::new(Mutex::new(false))
		}
	}
	
	/// creates a new tcp socket.
	#[allow(dead_code)]
	pub fn new(address: &'static str) -> Socket {
		let socket = Socket {
			writer   : Arc::new(Mutex::new(None)),
			reader   : Arc::new(Mutex::new(None)),
			queue    : Queue::new(1),
			onconnect: Event::new(),
			onread   : Event::new(),
			onerror  : Event::new(),
			onend    : Event::new(),
			reading  : Arc::new(Mutex::new(false))
		};
		let this = socket.clone();
		
		socket.queue.pause();
		socket.connect(address).then(move |result| {
			match result {
				Ok(stream) => {
					{
						let mut reader = this.reader.lock().unwrap();
						let mut writer = this.writer.lock().unwrap();
						*reader = Some(Reader::new(stream.try_clone().unwrap(), 4096));
						*writer = Some(Writer::new(stream.try_clone().unwrap()));
					}
					this.onconnect.emit((this.clone(), ));
					this.queue.resume();					
				}, Err(error) => {
						this.onerror.emit((this.clone(), error));
						this.queue.resume();
				}
			};
		}).async();	
		socket
	}
	
	/// to the onconnect event.
	#[allow(dead_code)]
	pub fn onconnect<F>(&self, action: F) -> Socket 
		where F: Fn((Socket, ))+Send+'static {
		self.onconnect.on(action);
		self.clone()
	}	
	
	/// connects and creates a tcp socket stream.
	#[allow(dead_code)]
	fn connect(&self, address: &'static str) -> Task<TcpStream, Error> {
		Task::new(move |task| {
			thread::spawn(move || {
				match TcpStream::connect(address) {
					Ok(stream) => task.call(Ok(stream)),
					Err(_)     => task.call(Err(Error::new("unable to connect to host")))
				}
			});
		})
	}
	
	/// reads from the stream while reading is true.
	fn read(&self) {
		let reader = self.reader.lock().unwrap();
		if let Some(ref reader) = *reader {
			let this = self.clone();
			reader.read().then(move |result| {
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
		else  {
			panic!()
		}
	}
}

impl ReadAsync for Socket {	
	fn ondata<F>(&self, action: F) -> Socket
		where F: Fn((Socket, Vec<u8>))+Send+'static  {
		self.onread.on(action);
		let this = self.clone();
		this.queue.clone().run(move |next| {
			next.call(());
			this.resume();
		});
		self.clone()
	}
	
	fn onerror<F>(&self, action: F) -> Socket
		where F: FnOnce((Socket, Error))+Send+'static {
		self.onerror.once(action);
		self.clone()
	}
	
	fn onend<F>(&self, action: F) -> Socket
		where F: FnOnce((Socket, ))+Send+'static {
		self.onend.once(action);
		let this = self.clone();
		this.queue.clone().run(move |next| {
			next.call(());
			this.resume();
		});
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

impl WriteAsync for Socket {
	fn write(&self, data: Vec<u8>) -> Task<(), Error> {
    let this = self.clone();
    Task::new(move |task| {
      let queue = this.queue.clone();
      queue.run(move |next| {
				let writer = this.writer.lock().unwrap();
				if let Some(ref writer) = *writer {
						writer.write(data).then(move |result| {
							match result {
								Ok(_)  => {
									task.call(Ok(()));
									next.call(());		
								},
								Err(_) => {
									task.call(Err(Error::new("unable to write to stream")));
									next.call(());				
								}
							}
						}).async();					
				}
      })
    })
	}
	
	fn end(&self) -> Task<(), Error> {
		let this = self.clone();
		Task::new(move |task| {
			let queue = this.queue.clone();
			queue.run(move |next| {
				this.pause();
				let clone      = this.clone();
				let mut writer = this.writer.lock().unwrap();
				let writer     = writer.take().unwrap();
				writer.end().then(move |result| {
					match result {
						Ok(_)  => {
							task.call(Ok(()));
							next.call(());
							clone.onend.emit((clone.clone(), ));			
						},
						Err(_) => {
							task.call(Err(Error::new("unable to end stream")));
							next.call(());						
						}
					}
				}).async();
			})
		})
	}
}