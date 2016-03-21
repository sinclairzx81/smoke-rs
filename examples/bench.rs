extern crate smoke;

fn main() {
  
  use smoke::io::ReadAsync;
  
  let file = std::fs::File::open("c:/input/video.mp4").unwrap();
  
  let buf = file.read_stream(65555)
                .read(0)
                .recv()
                .unwrap();
  
  println!("{:?}", buf);
  
  println!("finished");
}