extern crate smoke;


#[allow(unused_imports)]
use smoke::async::{Task, Scheduler};
use std::thread;


fn main() {
  
  let pool = Scheduler::threadpool(4);
  let handle = pool.spawn(move || {
  });
  
  println!("{:?}", handle.join().unwrap());
  
  thread::sleep_ms(6000);
}