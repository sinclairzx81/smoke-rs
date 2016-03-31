extern crate smoke;

use smoke::async::Stream;
use smoke::async::StreamSender;
use smoke::async::Task;

fn input() -> Stream<i32> {
  Stream::output(|sender| {
    (0..10).map(|n| sender.send(n)).last();
    Ok(())
  })
}

fn output() -> StreamSender<i32> {
  Stream::input(|receiver| {
    for n in receiver {
      println!("{}", n);
    } Ok(())
  })
}

fn pipe(input: Stream<i32>, output: StreamSender<i32>) -> Task<()> {
  Task::new(move |sender| {
    input.read()
         .into_iter()
         .map(|n| output.send(n))
         .last();
    sender.send(())
  })
}

fn main() {
    
    pipe(input(), output()).async(|_| {
      println!("pipe complete");
    }).wait().unwrap();
    
    println!("finished");
    
}