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

use super::action::ActionOnce;
use std::sync::mpsc::channel;
use std::thread::JoinHandle;
use std::thread;

pub struct Task<T, E> {
	closure: ActionOnce<ActionOnce<Result<T, E>>>
}
impl<T:Send+'static, E:Send+'static> Task<T, E> {
	
	#[allow(dead_code)]
	pub fn new<F>(closure: F) -> Task<T, E> 
		where F: FnOnce(ActionOnce<Result<T, E>>)+Send+'static {
		Task {
			closure: ActionOnce::new(closure)
		}
	}
	
	#[allow(dead_code)]
	pub fn then<F, U>(self, closure: F) -> Task<U, E>
		where F: FnOnce(Result<T, E>) -> U + Send + 'static,
			    U: Send + 'static {
		let (sender, receiver) = channel();
		let sender = sender.clone();
		self.closure.call(ActionOnce::new(move |result| {
			sender.send(result).unwrap();
		}));
		Task::<U, E>::new(move |resolver| {
			match receiver.recv() {
				Ok(result) => resolver.call(Ok(closure(result))), 
				Err(_)     => panic!()
			}
		})
	}
	
	#[allow(dead_code)]
	pub fn sync(self) -> Result<T, E> {
		let (sender, receiver) = channel();
		let sender = sender.clone();
		self.closure.call(ActionOnce::new(move |result| 
			sender.send(result).unwrap()
		)); receiver.recv().unwrap()
	}
	
	#[allow(dead_code)]
	pub fn async(self) -> JoinHandle<Result<T, E>> {
		thread::spawn(move || self.sync())
	}
	
	#[allow(dead_code)]
	pub fn race(tasks: Vec<Task<T, E>>)  -> Task<T, E> {
		Task::new(move |resolver| {
			let (sender, receiver) = channel();
			let len = tasks.len();
			for task in tasks {
				let sender = sender.clone();
				let _ = task.then(move |result| {
					sender.send(result).unwrap();
				}).async();
			}
			match receiver.recv() {
				Ok(result) => resolver.call(result), 
				Err(_)     => panic!()
			};
			for _ in (0..len-1) {
				let _ = receiver.recv().unwrap();
			}
		})
	}
	
	#[allow(dead_code)]
	pub fn series(tasks: Vec<Task<T, E>>) -> Task<Vec<T>, E> {
		Task::new(move |resolver| {
			let results:Vec<_> = 
				tasks.into_iter()
				.map(|task| task.sync())
				.collect();
			resolver.call(results
				.into_iter()
				.collect());
		})
	}
	
	#[allow(dead_code)]
	pub fn parallel(tasks: Vec<Task<T, E>>) -> Task<Vec<T>, E> {
		Task::new(move |resolver| {
			let results:Vec<_> = 
				tasks.into_iter()
				.map(|task| task.async())
				.map(|handle| handle.join().unwrap())
				.collect();
			resolver.call(results
				.into_iter()
				.collect());
		})
	}
}
