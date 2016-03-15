use smoke::async::Scheduler;
use std::time::Duration;
use std::thread;

#[test]
fn create_thread() {
  let _ = Scheduler::thread();
}

#[test]
fn create_thread_sequential_result() {
  let scheduler = Scheduler::thread();
  let handles:Vec<_> = 
    (0..10).map(|n| scheduler.run(move || {
      thread::sleep(Duration::from_millis(100));
      n
    })).collect();    
  let results: Vec<_> = 
    handles.into_iter()
           .map(|handle| handle.join().unwrap())
           .collect();  
  let mut idx = 0;
  for n in results {
    assert_eq!(n, idx);
    idx += 1;
  }
}

#[test]
fn create_thread_sleep_join() {
  let scheduler = Scheduler::thread();
  let handle = scheduler.run(|| {
    thread::sleep(Duration::from_millis(100));
    10
  });
  assert_eq!(handle.join().unwrap(), 10);
}

#[test]
fn create_threadpool() {
  let _ = Scheduler::thread();
}

#[test]
fn create_threadpool_sequential_result() {
  let scheduler = Scheduler::threadpool(4);
  let handles:Vec<_> = 
    (0..10).map(|n| scheduler.run(move || {
      thread::sleep(Duration::from_millis(100));
      n
    })).collect();    
  let results: Vec<_> = 
    handles.into_iter()
           .map(|handle| handle.join().unwrap())
           .collect();  
  let mut idx = 0;
  for n in results {
    assert_eq!(n, idx);
    idx += 1;
  }
}

#[test]
fn create_threadpool_sleep_join() {
  let scheduler = Scheduler::threadpool(4);
  let handle = scheduler.run(|| {
    thread::sleep(Duration::from_millis(100));
    10
  });
  assert_eq!(handle.join().unwrap(), 10);
}