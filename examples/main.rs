extern crate smoke;

use smoke::async::Stream;

fn numbers() -> Stream<i32> {
  Stream::new(|sender| {
    try! (sender.send(1) );
    try! (sender.send(2) );
    try! (sender.send(3) );
    sender.send(4)
  }) 
}

use std::fmt::Debug;
fn read<T: Debug + Send + 'static>(stream: Stream<T>) {
    for n in stream.read(0) {
      println!("{:?}", n);
    }
}

fn main() {
  // filter
  read(numbers().filter(|n| n % 2 == 0));
  // map
  read(numbers().map(|n| format!("num: {}", n)));
  // fold
  println!("{}", 
    numbers().fold(0, |p, c| p + c)
             .wait()
             .unwrap());
  // everything
  println!("{}", 
    numbers().filter(|n| n % 2 == 0)
             .map(|n| format!("{}", n))
             .fold(String::new(), |p, c| 
                    format!("{} {} and", p, c))
             .wait()
             .unwrap());
}
