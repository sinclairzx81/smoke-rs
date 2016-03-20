extern crate smoke;

use smoke::async::Task;

fn hello() -> Task<&'static str> {
  Task::delay(100).map(|_| "hello")
}

fn main() {
  println!("{:?}", hello().wait());
}
