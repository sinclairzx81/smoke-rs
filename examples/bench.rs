extern crate smoke;




// /// simple tcp server.
// fn server(local: &'static str) -> Stream<TcpStream> {
//   Stream::new(move |sender| {
//     let listener = TcpListener::bind(local).unwrap();
//     for stream in listener.incoming() {
//         try!(sender.send(stream.unwrap()))
//     } Ok(())
//   })
// }

// /// simple tcp client stream.
// fn client(remote: &'static str) -> Task<TcpStream> {
//   Task::new(move |sender| {
//     let stream = TcpStream::connect(remote).unwrap();
//     sender.send(stream)
//   })
// }





fn main() {
  
  use smoke::io::ReadAsync;
  
  let read = std::net::TcpStream::connect("towel.blinkenlights.nl:23").unwrap();
  //let read = std::fs::File::open("c:/input/export.json").unwrap();
  //let read = std::io::empty();
  
  for line in read.read_line_stream().read(0) {
    println!("{}", line);
  }
  println!("finished");
}