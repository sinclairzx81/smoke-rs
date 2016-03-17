extern crate smoke;

use smoke::async::{Task, Scheduler};

fn main() {
  let scheduler = Scheduler::new(4);
  let task = Task::scheduled(scheduler, |sender| {
    sender.send(100)
  });
  
  println!("{}", task.wait().unwrap());
}
