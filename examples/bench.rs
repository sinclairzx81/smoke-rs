extern crate smoke;


fn main() {
  
   use smoke::io::ReadAsync;
   
   let read = std::io::empty();
   
   let task = read.read_to_end_task().async(|result| {
     // bytes available here.
   });
  
  println!("finished");
}