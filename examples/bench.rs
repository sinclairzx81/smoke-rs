extern crate smoke;

use smoke::async::{Task, StreamSender, Stream};
use std::net::{TcpListener, TcpStream};
use std::io::prelude::*;
use std::io::BufReader;

//----------------------------------
// Client 
//----------------------------------
struct Client {
  input  : Stream<String>,
  output : StreamSender<String>
}
impl Client {
  fn new(stream: TcpStream) -> Client {
    let     read  = stream.try_clone().unwrap();
    let mut write = stream.try_clone().unwrap();
    Client {
      input: Stream::output(move |sender| {
          let mut reader = BufReader::new(read);
          let mut buf    = String::new();
          while reader.read_line(&mut buf).unwrap() > 0 {
              try!(sender.send(buf.clone()));
              buf.clear();
          } Ok(())  
      }),
      output: Stream::<String>::input(move |receiver| {
        for message in receiver {
          let mut bytes = message.as_bytes();
          let _ = write.write(&mut bytes).unwrap();
        }
      })
    }
  }
}
//----------------------------------
// Server Stream.
//----------------------------------
fn server(addrs: &'static str) -> Stream<Client> {
  Stream::output(move |sender| {
    let listener = TcpListener::bind(addrs).unwrap();
    for stream in listener.incoming() {
      let stream = stream.unwrap();
      let client = Client::new(stream);
      try!(sender.send(client));
    } Ok(())
  })
}


fn main() {
    //-------------------------
    // small echo server.
    //-------------------------
    for client in server("localhost:5000").read() {
      Task::new(move |sender| {
        for message in client.input.read() {
          client.output.send(message).unwrap();
        } sender.send(())
      }).async(|_| println!("socket closed"));
    }
}