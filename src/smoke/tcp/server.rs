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

use super::super::{Error};
use super::super::async::Event;
use super::socket::Socket;
use std::net::{TcpListener};
use std::thread;

pub struct Server {
	onsocket: Event<Socket>,
	onerror : Event<Error>
}
impl Clone for Server {
	fn clone(&self) -> Server {
		Server {
			onsocket: self.onsocket.clone(),
			onerror : self.onerror.clone()
		}
	}
}
impl Server {
	#[allow(dead_code)]
	pub fn new<F>(onsocket: F) -> Server 
		where F: Fn(Socket)+Send+'static {
		Server {
			onsocket: { 
				let event = Event::new();
				event.on(onsocket);
				event
			},
			onerror : Event::new()
		}
	}
	#[allow(dead_code)]
	pub fn listen(&self, address: &'static str) {
		let this = self.clone();
		thread::spawn(move || {
			let listener = TcpListener::bind(address).unwrap();
			for stream in listener.incoming() {
				match stream {
					Ok(stream) => {
						let socket = Socket::from_stream(stream);
						let this = this.clone();
						thread::spawn(move || {
							this.onsocket.emit(socket);
						});
					}
					Err(_) => this.onerror.emit(Error::new("unable to accept socket"))
				}
			}			
		});
	}
}