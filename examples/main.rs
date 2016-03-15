extern crate smoke;

#[allow(unused_imports)]
use smoke::async::{Task, Stream, Scheduler};

fn helloworld() -> Task<&'static str> {
  Task::new(move |sender| {
      sender.send("hello world")
  })
}

fn main() {
  println!("{}", helloworld()
                .async()
                .join()
                .unwrap());
}