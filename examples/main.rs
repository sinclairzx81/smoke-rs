extern crate smoke;

use smoke::async::Task;
use smoke::async:: {
  SyncScheduler,
  ThreadScheduler,
  ThreadPoolScheduler
};

fn hello() -> Task<&'static str> {
  Task::delay(1).map(|_| "hello")
}

fn main() {
   // runs the task synchronously.
   let scheduler = SyncScheduler;
   let result    = hello().schedule(scheduler).wait();
   
   // creates a new thread for each task run.
   let scheduler = ThreadScheduler;
   let result    = hello().schedule(scheduler).wait();
      
   // schedules the task to be run on a 
   // threadpool, in this example, there
   // are 8 available threads.
   let scheduler = ThreadPoolScheduler::new(8);
   let result    = hello().schedule(scheduler).wait();   
}
