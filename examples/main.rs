extern crate smoke;

#[allow(unused_imports)]
use smoke::async::{Task};

fn helloworld() -> Task<&'static str> {
  Task::new(move |sender| {
    let _ = Task::delay(1000).async(move |_| {
      sender.send("hello world")
    }).wait(); Ok(())
  })
}

fn main() {
  
  println!("{:?}", helloworld().sync().unwrap())
  
  
 
}