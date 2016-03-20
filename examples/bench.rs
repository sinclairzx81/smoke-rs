extern crate smoke;

use smoke::async::Task;
use smoke::async::Stream;
use std::thread;
use std::net::{TcpListener, TcpStream};

/// simple tcp server.
fn server(local: &'static str) -> Stream<TcpStream> {
  Stream::new(move |sender| {
    let listener = TcpListener::bind(local).unwrap();
    for stream in listener.incoming() {
        try!(sender.send(stream.unwrap()))
    } Ok(())
  })
}

/// simple tcp client stream.
fn client(remote: &'static str) -> Task<TcpStream> {
  Task::new(move |sender| {
    let stream = TcpStream::connect(remote).unwrap();
    sender.send(stream)
  })
}

fn main() {
  
  // server handler....
  thread::spawn(|| {
    for stream in server("localhost:5000").read(0) {
        println!("server: new stream");
    }   
  });
  
  // clients..
  let tasks = (0..50).map(|n| client("localhost:5000")).collect::<Vec<_>>();
  Task::all(4, tasks).async(|streams| {
     println!("clients connected: {:?}", streams);
  }).wait();
  
  thread::sleep_ms(100);

}