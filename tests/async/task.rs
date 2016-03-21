
use smoke::async::Task;




#[test]
fn create() {
  let _ = Task::new(|sender| { sender.send(10) });
}

#[test]
fn wait() {
  let task = Task::new(|sender| { sender.send(10) });
  assert_eq!(task.wait().unwrap(), 10);
}

#[test]
fn await() {
  let task = Task::new(|sender| { sender.send(10) });
  assert_eq!(task.async(|result| result.unwrap()).wait().unwrap(), 10);
}

#[test]
fn wait_then() {
  fn increment(value: i32) -> Task<i32> {
    Task::delay(10).map(move |_| value + 1) 
  }
  assert_eq!(4, increment(0)
                .then(increment)
                .then(increment)
                .then(increment)
                .wait()
                .unwrap());
}

#[test]
fn async_then() {
  fn increment(value: i32) -> Task<i32> {
    Task::delay(1).map(move |_| value + 1) 
  }
  let result = increment(0)
      .then(increment)
      .then(increment)
      .then(increment)
      .async(|result| {
        assert_eq!(4, result.unwrap());
        123
      }).wait();
   assert_eq!(123, result.unwrap());
}

#[test]
fn then_result_with_panic() {
  fn increment(value: i32) -> Task<i32> {
    Task::delay(1).map(move |_| value + 1) 
  }
  fn boom(_: i32) -> Task<i32> {
    Task::new(|_| {
      panic!("boom")
    })
  }
  match increment(0)
    .then(increment)
    .then(boom)
    .then(increment)
    .wait() {
       Ok(_) => panic!("unexpected result"),
       Err(_) => {/* ok */}         
  };
}
#[test]
#[should_panic]
fn wait_no_result_match() {
  let task = Task::<i32>::new(|_| { Ok(()) });
  task.wait().unwrap();
}

#[test]
#[should_panic]
fn wait_no_result_unwrap() {
  let task = Task::<i32>::new(|_| { Ok(()) });
  task.wait().unwrap();
}