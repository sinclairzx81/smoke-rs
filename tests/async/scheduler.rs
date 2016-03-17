use smoke::async::Scheduler;
use std::time::Duration;
use std::thread;

#[test]
fn create_thread() {
  let _ = Scheduler::new(8);
}

#[test]
fn create_thread_sequential_result() {
  let scheduler = Scheduler::new(8);
  let handles:Vec<_> = 
    (0..10).map(|n| scheduler.run(move || {
      thread::sleep(Duration::from_millis(100));
      n
    })).collect();    
  let results: Vec<_> = 
    handles.into_iter()
           .map(|handle| handle.wait().unwrap())
           .collect();  
  let mut idx = 0;
  for n in results {
    assert_eq!(n, idx);
    idx += 1;
  }
}

#[test]
fn create_thread_sleep_join() {
  let scheduler = Scheduler::new(8);
  let handle = scheduler.run(|| {
    thread::sleep(Duration::from_millis(100));
    10
  }); assert_eq!(handle.wait().unwrap(), 10);
}
