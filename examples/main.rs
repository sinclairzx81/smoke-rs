extern crate smoke;
extern crate threadpool;

#[allow(unused_imports)]
use smoke::async::{Task, Scheduler};


fn main() {
  let pool = Scheduler::new();
  let handle = pool.spawn(move || {
    
  });
  
  handle.join();
}