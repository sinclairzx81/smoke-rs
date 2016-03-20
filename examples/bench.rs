extern crate smoke;

use std::io::Read;
use smoke::async::{Task, Stream};
use std::net::{TcpListener, TcpStream};

use std::sync::{Arc, Mutex};
struct ReadAsync<R> {
  reader: Arc<Mutex<R>>
}
impl<R> ReadAsync<R> where R: Read + Send + 'static {
  fn new(reader: R) -> ReadAsync<R> {
    ReadAsync { reader: Arc::new(Mutex::new(reader)) }
  }
  /// reads bytes from this buffer.
  pub fn read(&self) -> Task<(usize, [u8; 16384])> {
    let reader = self.reader.clone();
    Task::new(move |sender| {
      let mut reader = reader.lock().unwrap();
      let mut buf    = [0; 16384];
      let size       = reader.read(&mut buf).unwrap();
      sender.send((size, buf))
    })
  }
  /// converts this read into a stream.
  pub fn stream(&self) -> Stream<(usize, [u8; 16384])> {
    let reader = self.reader.clone();
    Stream::new(move |sender| {
      let mut reader = reader.lock().unwrap();
      loop {
        let mut buf = [0; 16384];
        let size = reader.read(&mut buf).unwrap();
        if size > 0 {
          try!(sender.send((size, buf)));
        } else {
          break;
        }
      } Ok(())
    })
  }  
}

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
  
  // starwars test
  client("towel.blinkenlights.nl:23").async(|stream| {
    println!("connected");
    for (size, _) in ReadAsync::new(stream.unwrap()).stream().read(0) {
      println!("read: {:?}", size); 
    }
  }).wait();
  
  println!("finished");
}