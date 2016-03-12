mod smoke;
use smoke::async::Stream;

#[derive(Debug)]
enum Foo {
  Number(i32),
  Word(&'static str)
}

fn numbers() -> Stream<Foo> {
  Stream::new(|sender| {
    try! (sender.send(Foo::Number(1)) );
    try! (sender.send(Foo::Number(2)) );
    try! (sender.send(Foo::Number(3)) );
    try! (sender.send(Foo::Number(4)) );
    Ok(())
  }) 
}

fn words() -> Stream<Foo> {
  Stream::new(|sender| {
      "the quick brown fox jumps over the lazy dog"
        .split(" ")
        .map(|n| sender.send(Foo::Word(n)))
        .last()
        .unwrap()
  }) 
}

fn main() {
  // mux
  let stream = Stream::mux(vec![
      numbers(), 
      words()
      ]);
  
  // demux
  for foo in stream.recv() {
      match foo {
        Foo::Number(n) => 
          println!("number -> {}", n),
        Foo::Word(n) => 
          println!("word -> {}", n)
      }
  }
}