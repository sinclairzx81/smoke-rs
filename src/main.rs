
mod smoke;
use smoke::async::{Task, Stream};

//----------------------------------
// generates async sequence
//----------------------------------
fn sequence(start: i32, end: i32) -> Stream<i32> {
    Stream::new(move |sender| {
        for n in start..end {
            std::thread::sleep(std::time::Duration::from_millis(1));
            try !(sender.send(n)); // emit
        } Ok(())
    })
}

// generates async sequence
fn sequence2(start: i32, end: i32) -> Stream<&'static str> {
    sequence(start, end).map(|_| "hello")
}

// generates async sequence...

fn main() {
   let task1 = Task::<i32, ()>::new(|| {
        // merge streams into a single stream...
        for n in Stream::merge(vec![ sequence(0,  10).filter(|n| n % 2 == 0),
                                     sequence(10, 20).filter(|n| n % 2 == 0), 
                                     sequence(20, 30).filter(|n| n % 2 == 0) ]).recv() {
            println!("{}", n);
        } Ok(1)
   });
   
   let task2 = Task::<i32, ()>::new(|| {
        let result = Stream::merge(vec![ sequence(0,  2) ])
                            .recv()
                            .into_iter()
                            .collect::<Vec<i32>>();
        println!("{:?}", result);
        Ok(2)                
   });
   
   let _ = Task::all(vec![task1, task2]).async(|result| {
       println!("{:?}", result);
       
   }).join();
   
   
   
   println!("ended");
}