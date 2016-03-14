
use smoke::async::Task;

#[test]
fn create() {
  let _ = Task::new(|sender| { sender.send(10) });
}

#[test]
fn sync() {
  let task = Task::new(|sender| { sender.send(10) });
  assert_eq!(task.sync().unwrap(), 10);
}

#[test]
fn async() {
  let task = Task::new(|sender| { sender.send(10) });
  assert_eq!(task.async().join().unwrap(), 10);
}

#[test]
fn sync_then() {
  let task = Task::new(|sender| { sender.send(10) })
                 .then(|n| Task::new(move |sender| sender.send(n + 10)));
  assert_eq!(task.sync().unwrap(), 20);
}

#[test]
fn async_then() {
  let task = Task::new(|sender| { sender.send(10) })
                .then(|n| Task::new(move |sender| sender.send(n + 10)));
  assert_eq!(task.async().join().unwrap(), 20);
}

#[test]
#[should_panic]
fn sync_panic() {
  let task = Task::<i32>::new(|_| { panic!() });
  task.sync().unwrap();
}

#[test]
fn async_panic() {
  let task = Task::<i32>::new(|_| { panic!() });
  match task.async().join() {
    Ok(_)  => panic!(),
    Err(_) => assert!(true)
  }
}

#[test]
fn sync_no_result_match() {
  let task = Task::<i32>::new(|_| { Ok(()) });
  match task.sync() {
    Ok(_)  => panic!(),
    Err(_) => assert!(true)
  }
}

#[test]
#[should_panic]
fn sync_no_result_unwrap() {
  let task = Task::<i32>::new(|_| { Ok(()) });
  task.sync().unwrap();
}

#[test]
fn async_no_result_match() {
  let task = Task::<i32>::new(|_| { Ok(()) });
  match task.async().join() {
    Ok(_)  => panic!(),
    Err(_) => assert!(true)
  }
}

#[test]
#[should_panic]
fn async_no_result_unwrap() {
  let task = Task::<i32>::new(|_| { Ok(()) });
  task.async().join().unwrap();
}