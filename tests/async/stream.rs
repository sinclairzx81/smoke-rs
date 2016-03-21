use smoke::async::{Stream, ToStream};

#[test]
fn create() {
  let _ = Stream::new(|sender| sender.send(10));
}

#[test]
fn read() {
  let stream = Stream::new(|sender| {
    try!(sender.send(1));
    try!(sender.send(2));
    sender.send(3) 
  });
  let receiver = stream.read(0);
  assert_eq!(1, receiver.recv().unwrap());
  assert_eq!(2, receiver.recv().unwrap());
  assert_eq!(3, receiver.recv().unwrap());
}

#[test]
fn merge() {
  fn stream() -> Stream<i32> {
    Stream::new(|sender| {
      try!(sender.send(1));
      try!(sender.send(1));
      sender.send(1) 
    })
  }
  let mut acc = 0;
  let s = Stream::merge(vec![
    stream(), 
    stream(), 
    stream()
    ]);
  for n in s.read(0) {
    acc = acc + n;
  } assert_eq!(9, acc);
}

#[test]
fn map() {
  fn stream() -> Stream<i32> {
    Stream::new(|sender| {
      try!(sender.send(1));
      try!(sender.send(1));
      sender.send(1) 
    })
  }
  let mut acc = 0;
  let s = stream().map(|n| n+1);
  for n in s.read(0) {
    acc = acc + n;
  } assert_eq!(6, acc);
}

#[test]
fn filter() {
  fn stream() -> Stream<i32> {
    Stream::new(|sender| {
      try!(sender.send(1));
      try!(sender.send(2));
      sender.send(1) 
    })
  }
  let mut acc = 0;
  let s = stream().filter(|n| *n == 1);
  for n in s.read(0) {
    acc = acc + n;
  } assert_eq!(2, acc);
}

#[test]
fn fold() {
  fn stream() -> Stream<i32> {
    Stream::new(|sender| {
      try!(sender.send(1));
      try!(sender.send(1));
      sender.send(1) 
    })
  }
  let task = stream().fold(0, |p, c| p + c);
  assert_eq!(3, task.wait().unwrap());
}

#[test]
fn to_stream() {
  let mut idx = 0;
  for n in (0 .. 10).to_stream().read(0) {
    assert_eq!(n, idx); idx += 1;
  }
}