mod smoke;
use smoke::async::Stream;

fn numbers() -> Stream<i32> {
  Stream::new(|sender| {
      (0..10).map(|n| sender.send(n))
             .last()
             .unwrap()
  }) 
}

fn main() {
  for n in numbers().filter(|n| n % 2 == 0) // only even numbers
                    .map   (|n| n * 2)      // multiple them by 2.
                    .recv  () {
      println!("{}", n);
  }
}