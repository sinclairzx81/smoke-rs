mod smoke;
use smoke::async::Task;
use std::thread;
use std::time::Duration;

fn add(a: i32, b: i32) -> Task<i32, ()> {
  Task::new(move || {
    thread::sleep(Duration::from_secs(1));
    Ok(a + b)
  })
}

fn main() {
   let result = Task::all(vec![
     add(10, 20),
     add(20, 30),
     add(30, 40)
   ]).sync().unwrap(); 
   
   // [30, 50, 70]
   println!("{:?}", result); 
}