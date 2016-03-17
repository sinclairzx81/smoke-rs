
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
  let task = Task::new(|sender| { sender.send(10) })
                 .then(|n| Task::new(move |sender| sender.send(n.unwrap() + 10)));
  assert_eq!(task.wait().unwrap(), 20);
}

#[test]
fn async_then() {
  let task = Task::new(|sender| { sender.send(10) })
                .then(|n| Task::new(move |sender| sender.send(n.unwrap() + 10)));
  assert_eq!(task.async(|result| result.unwrap()).wait().unwrap(), 20);
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