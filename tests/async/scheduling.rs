use smoke::async::Task;
use smoke::async::{
  ThreadScheduler, 
  SyncScheduler, 
  ThreadPoolScheduler
};
/// creates a task that will pass.
fn create_ok_task() -> Task<i32> {
  Task::delay(1).map(|_| 1)
}
/// creates a task that will panic.
fn create_panic_task() -> Task<i32> {
  Task::delay(1).then(|_| Task::new(|_| {
    panic!("boom");
  }))
}
///------------------------------------
/// SyncScheduler
///------------------------------------
#[test]
fn sync_scheduler_create() {
  let _ = SyncScheduler;
}
#[test]
fn sync_scheduler_run_ok_task() {
  let scheduler = SyncScheduler;
  let task      = create_ok_task();
  match task.schedule(scheduler).wait() {
    Ok(result) => assert_eq!(1, result),
    Err(_) => {/* .. */}
  }
}
#[test]
#[should_panic]
fn sync_scheduler_run_panic_task() {
  let scheduler = SyncScheduler;
  let task      = create_panic_task();
  match task.schedule(scheduler).wait() {
    Ok(result) => assert_eq!(1, result),
    Err(_) => {/* .. */}
  }
}
#[test]
fn sync_scheduler_run_from_task() {
  let scheduler = SyncScheduler;
  let task      = create_ok_task();
  match task.schedule(scheduler).wait() {
    Ok(result) => assert_eq!(1, result),
    Err(_) => {/* .. */}
  }
}

///------------------------------------
/// ThreadScheduler
///------------------------------------
#[test]
fn thread_scheduler_create() {
  let _ = ThreadScheduler::new();
}
#[test]
fn thread_scheduler_run_ok_task() {
  let scheduler = ThreadScheduler::new();
  let task      = create_ok_task();
  match task.schedule(scheduler).wait() {
    Ok(result) => assert_eq!(1, result),
    Err(_) => {/* .. */}
  }
}
#[test]
fn thread_scheduler_run_panic_task() {
  let scheduler = ThreadScheduler::new();
  let task      = create_panic_task();
  match task.schedule(scheduler).wait() {
    Ok(result) => assert_eq!(1, result),
    Err(_) => {/* .. */}
  }
}
#[test]
fn thread_scheduler_run_from_task() {
  let scheduler = ThreadScheduler::new();
  let task      = create_ok_task();
  match task.schedule(scheduler).wait() {
    Ok(result) => assert_eq!(1, result),
    Err(_) => {/* .. */}
  }
}

///------------------------------------
/// ThreadPoolScheduler
///------------------------------------
#[test]
fn thread_pool_scheduler_create() {
  let _ = ThreadPoolScheduler::new(4);
}
#[test]
fn thread_pool_scheduler_run_ok_task() {
  let scheduler = ThreadPoolScheduler::new(4);
  let task      = create_ok_task();
  match task.schedule(scheduler).wait() {
    Ok(result) => assert_eq!(1, result),
    Err(_) => {/* .. */}
  }
}
#[test]
fn thread_pool_scheduler_run_panic_task() {
  let scheduler = ThreadPoolScheduler::new(4);
  let task      = create_panic_task();
  match task.schedule(scheduler).wait() {
    Ok(result) => assert_eq!(1, result),
    Err(_) => {/* .. */}
  }
}
#[test]
fn thread_pool_scheduler_run_from_task() {
  let scheduler = ThreadPoolScheduler::new(4);
  let task      = create_ok_task();
  match task.schedule(scheduler).wait() {
    Ok(result) => assert_eq!(1, result),
    Err(_) => {/* .. */}
  }
}