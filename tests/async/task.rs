
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
  assert_eq!(task.async(|result| result.unwrap()).wait().unwrap(), 10);
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
  assert_eq!(task.async(|result| result.unwrap()).wait().unwrap(), 20);
}


#[test]
#[should_panic]
fn sync_no_result_match() {
  let task = Task::<i32>::new(|_| { Ok(()) });
  task.sync().unwrap();
}

#[test]
#[should_panic]
fn sync_no_result_unwrap() {
  let task = Task::<i32>::new(|_| { Ok(()) });
  task.sync().unwrap();
}

/* 
task not completing on async no result. need to 
either detect panic, or check the drop on the 
sender.  
#[test]
#[should_panic]
fn async_no_result_match() {
  let task = Task::<i32>::new(|_| { Ok(()) });
  task.async(|result| result.unwrap()).wait().unwrap();
}
*/
/*
#[test]
#[should_panic]
fn async_no_result_unwrap() {
  let task = Task::<i32>::new(|_| { Ok(()) });
  task.async(|result| result.unwrap()).wait().unwrap();
}
*/