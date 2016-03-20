extern crate smoke;

use smoke::async::Task;

//--------------------------------
// IMPLEMENTATION
//--------------------------------
fn hello() -> Task<&'static str> {
  Task::delay(1).map(|_| "hello")
}

fn main() {
  println!("{:?}", hello().wait())
}
