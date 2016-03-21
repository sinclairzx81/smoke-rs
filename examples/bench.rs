extern crate smoke;

use smoke::async::Stream;

fn runtime() -> Stream<()> {
  Stream::new(move |sender| {
    loop {
      // here, this loop will continue to 
      // send as long as the receiver is 
      // owned. If the receiver drops, then
      // the call to send() will result in 
      // Err(). In case of either Ok() or
      // Err(), the calling thread will 
      // exit this closure and terminate
      // gracefully.
      try!(sender.send(()));
    }
  })
}

fn main() {
  
  // calling read() returns a mpsc
  // receiver which is becomes owned 
  // by the for loop.
  for _ in runtime().read(0) {
    println!("got one");
    break; // no more please...
  } 
  
  // here, the receiver is no longer,
  // owned, and the sending thread
  // exits gracefully.
  
  println!("finished");
}