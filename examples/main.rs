extern crate smoke;

use smoke::async::Task;

// simple add function.
fn add(a: i32, b: i32) -> Task<i32> {
  Task::new(move |sender| {
    sender.send(a + b)
  })
}
// simple format function.
use std::fmt::Debug;
fn format<T>(t: T) -> Task<String> where
   T:Debug + Send + 'static {
   Task::new(move |sender| {
      let n = format!("result is: {:?}", t);
      sender.send(n)
   })
}

fn hello() -> Task<&'static str> {
  Task::new(|sender| {
    sender.send("hello world!!")
  })
}
fn main() {
   
   // add two numbers
   let result = add(1, 2).wait();
   println!("{:?}", result.unwrap()); 
   
   // wait 1 second.
   // add two numbers.
   let result = Task::delay(1000)
                .then(|_| add(10, 20))
                .wait();
                
   println!("{:?}", result.unwrap()); 
              
   // wait 1 second.
   // add two numbers.
   // add 3.
   // format.
   let result = Task::delay(1000)
                .then(|_|   add(100, 200))
                .then(|res| add(res.unwrap(), 3))
                .then(|res| format(res.unwrap()))
                .wait();
   
   println!("{:?}", result.unwrap());
}