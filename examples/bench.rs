extern crate smoke;

#[allow(unused_imports)]
use smoke::async::Task;

fn main() {
  
  fn increment(value: i32) -> Task<i32> {
    Task::delay(10).map(move |_| value + 1) 
  }
  
  fn boom(value: i32) -> Task<i32> {
    Task::new(|sender| {
      panic!("boom")
    })
  }
  
  println!("{:?}", increment(0)
                .then(increment)
                .then(increment)
                .then(increment)
                .wait());
                
   println!("finished ok");
}