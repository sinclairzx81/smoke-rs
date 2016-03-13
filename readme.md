#smoke-rs

Concurrency primitives for the Rust programming language.

## overview

This library provides task / stream primitives to help orchestrate concurrency in Rust.  

* [Task&lt;T, E&gt;](#task)
  * [Creating tasks](#creating_tasks)
  * [Run task synchronously](#run_task_synchronously)
  * [Run task asynchronously](#run_task_asynchronously)
  * [Run tasks in parallel](#run_tasks_in_parallel)
* [Stream&lt;T&gt;](#stream)
  * [Creating streams](#creating_streams)
  * [Reading streams](#reading_streams)
  * [Merging streams](#merging_streams)
  * [Mapping streams](#mapping_streams)
* [Examples](#examples)
  * [Processing files](#processing_files)

<a name='task'></a>
## Task&lt;T, E&gt;

A Task encapsulates a single operation and provides a means to run that operation 
synchronously or asynchronously. A task achieves this by providing the caller a .sync() 
or .async() method which the caller can use to resolve a std::Result&lt;T, E&gt;.

<a name='creating_tasks'></a>
### Creating Tasks

The following will create a task. It is important to note that a task
will not execute immediately and the caller is expected to call .sync() or .async()
on the task to execute the tasks body.

```rust
mod smoke;
use smoke::async::Task;

fn main() {
    let task = Task::<i32, i32>::new(|| {
        println!("inside the task");
        Ok(123)
    });
}
```

<a name='run_task_synchronously'></a>
### Run task synchronously

The following creates a task which resolves a Result<&lt;i32, i32&gt; and executes it synchronously.

```rust
mod smoke;
use smoke::async::Task;

fn main() {
    let task = Task::<i32, i32>::new(|| {
        println!("inside the task");
        Ok(123)
    });
    
    let number = task.sync().unwrap();
}
```

<a name='run_task_asynchronously'></a>
### Run task asynchronously

The following creates a task which resolves a Result&lt;i32, i32&gt; and executes it asynchronously. Internally,
the task will be executed within a new thread. .async() returns a JoinHandle&lt;Result&lt;i32, i32&gt;&gt; a caller
can .join() later to obtain the result later.

```rust
mod smoke;
use smoke::async::Task;

fn main() {
    let task = Task::<i32, i32>::new(|| {
        println!("inside the task");
        Ok(123)
    });
    
    let handle = task.async();
    // ...
    let result = handle.join().unwrap();
}
```
optionally, the result can be obtained on a continuation. 

```rust
fn main() {
    let task = Task::<i32, i32>::new(|| {
        println!("inside the task");
        Ok(123)
    });
    
    let handle = task.async_then(|result| {
      println!("result: {}", result.unwrap());
      
      // optionally return
    });
    
    // ... do other work.
    let _ = handle.join().unwrap();
}
```

<a name='run_tasks_in_parallel'></a>
### Run tasks in parallel

Tasks can be run in parallel by calling Task::all(). The all() method wraps the inner tasks in a outer task
that the caller can use to obtain the results. In this regard, a vector of Task&lt;T, E&gt; will map to a new
task of type Task&lt;Vec&lt;T&gt;, E&gt;.

```rust
mod smoke;
use smoke::async::Task;
use std::thread;
use std::time::Duration;

fn add(a: i32, b: i32) -> Task<i32, ()> {
  Task::new(move || {
    thread::sleep(Duration::from_secs(1));
    Ok(a + b)
  })
}

fn main() {
   let result = Task::all(vec![
     add(10, 20),
     add(20, 30),
     add(30, 40)
   ]).sync().unwrap(); 
   
   // [30, 50, 70]
   println!("{:?}", result); 
}
```

<a name='stream'></a>
## Stream&lt;T&gt;

Stream&lt;T&gt; provides a means to generate async sequences from 
which a caller may listen to. Internally, Stream&lt;T&gt; abstracts 
mpsc channels and provides some composability methods to aid in data 
flow.


<a name='creating_streams'></a>
## Creating Streams

The following creates a simple sequence of numbers. 

```rust
mod smoke;
use smoke::async::Stream;

fn main() {
  let stream = Stream::new(|sender| {
      println!("inside stream");
      try! (sender.send(1) );
      try! (sender.send(2) );
      try! (sender.send(3) );
      try! (sender.send(4) );
      Ok(())
  });
}
```

<a name='reading_streams'></a>
## Reading Streams

A stream can be read with the streams .sync() and .async() functions. Internally,
.sync() and .async() functions map to mpsc SyncSender and Sender respectively. 

Consider the following stream.

```rust
mod smoke;
use smoke::async::Stream;

fn numbers() -> Stream<i32> {
    Stream::new(|sender| {
      for n in 0..5 {
          try! ( sender.send(n) );
          println!("sent: {}", n);
      }
      Ok(())
    })
}
```

The code below will call the stream with .async(). This will start reading the entire stream from
start to finish. Because the reader is sleeping the thread (emulating some contention), the .async()
function will internally buffer streamed data until the reader gets around to reading it.

```rust
use std::thread;
use std::time::Duration;

fn main() {
  for n in numbers().async() {
      thread::sleep(Duration::from_secs(1));
      println!("recv: {}", n);
  }
}

// output:
// sent: 0
// sent: 1
// sent: 2
// sent: 3
// sent: 4
// recv: 0
// recv: 1
// recv: 2
// recv: 3
// recv: 4
```

Often, buffering is not desirable instances where the stream may produce large amounts of data (for example, streaming
data from a large file). In these instances, callers can use the .sync(bound) function. Unlike the .async() function which
will buffer infinately, .sync(bound) will buffer items up to the specified bound. If the bound is exceeded, the streams sender
will block.

With this, callers can read in lock-stop with the stream.

```rust
use std::thread;
use std::time::Duration;

fn main() {
  for n in numbers().sync(0) {
      thread::sleep(Duration::from_secs(1));
      println!("sent: {}", n);
  }
}

// output:
// sent: 0
// recv: 0
// sent: 1
// recv: 1
// sent: 2
// recv: 2
// sent: 3
// recv: 3
// sent: 4
// recv: 4
```

<a name='merging_streams'></a>
## Merging streams

Multiple streams of the same type can be merged into a single stream. The following creates two 
distinct streams (numbers and words), merges them into a single stream and reads.

```rust
mod smoke;
use smoke::async::Stream;

#[derive(Debug)]
enum Item {
  Number(i32),
  Word(&'static str)
}

fn numbers() -> Stream<Item> {
  Stream::new(|sender| {
    try! (sender.send(Item::Number(1)) );
    try! (sender.send(Item::Number(2)) );
    try! (sender.send(Item::Number(3)) );
    try! (sender.send(Item::Number(4)) );
    Ok(())
  }) 
}

fn words() -> Stream<Item> {
  Stream::new(|sender| {
      "the quick brown fox jumps over the lazy dog"
        .split(" ")
        .map(|n| sender.send(Item::Word(n)))
        .last()
        .unwrap()
  }) 
}

fn main() {
  // mux
  let stream = Stream::all(vec![
    numbers(),
    words()
  ]);
  // demux
  for item in stream.sync(0) {
    match item {
      Item::Word(word)     => println!("word: {:?}", word),
      Item::Number(number) => println!("number: {:?}", number)
    }
  }
}
```

<a name='mapping_streams'></a>
## Mapping streams

Sometimes, it may be desirable to map one stream to another stream. Streams provide a .map() function
a caller can use to map one stream into another. The following example builds on the merge example, mapping
different streams into a unified type. Useful for integration.

```rust
mod smoke;
use smoke::async::Stream;

#[derive(Debug)]
enum Item {
  Number(i32),
  Word(&'static str)
}

fn numbers() -> Stream<i32> {
  Stream::new(|sender| {
    try! (sender.send(1) );
    try! (sender.send(2) );
    try! (sender.send(3) );
    try! (sender.send(4) );
    Ok(())
  }) 
}

fn words() -> Stream<&'static str> {
  Stream::new(|sender| {
      "the quick brown fox jumps over the lazy dog"
        .split(" ")
        .map(|n| sender.send(n))
        .last()
        .unwrap()
  }) 
}


fn main() {
  // mux
  let stream = Stream::all(vec![
    numbers().map(|n| Item::Number(n)),
    words().map  (|n| Item::Word(n))
  ]);
  // demux
  for item in stream.sync(0) {
    match item {
      Item::Word(word)     => println!("word: {:?}", word),
      Item::Number(number) => println!("number: {:?}", number)
    }
  }
}
```

<a name='examples'></a>
## Examples

<a name='processing_files'></a>
### Processing files

The program below will demonsrates a simple file processor using Tasks and Streams. The filestream()
function will opens a file as a Stream, scan() scans the files bytes and totals the number of bytes read, given
as a Task.

```rust
mod smoke;
use smoke::async::{Task, Stream};
use std::fs::File;
use std::io::prelude::*;

//------------------------------------
// filestream(): returns a filestream.
//------------------------------------
fn filestream(filename: &'static str) -> Stream<(usize, [u8; 16384])> {
  let mut file = File::open(filename).unwrap();
  Stream::new(move |sender| {
    println!("filestream: {}", filename);
    loop {
      let mut buf = [0; 16384];
      let size = file.read(&mut buf).unwrap();
      if size > 0 {
        try!(sender.send((size, buf)));
      } else {
        break;
      }
    } Ok(())
  })
}

//--------------------------------------
// scan(): returns the number of bytes.
//--------------------------------------
fn scan(filename: &'static str) -> Task<usize, ()> {
  Task::new(move || {
    println!("scan: {}", filename);
    let mut count = 0;
    for (size, _) in filestream(filename).sync(0) {
      count = count + size;
    } Ok(count)
  })
}

fn main() {
  println!("begin");
  let result = Task::all(vec![
    scan("file1.dat"),
    scan("file2.dat"),
    scan("file3.dat"),
    scan("file4.dat"),
    scan("file5.dat")
    // ... more
  ]).sync().unwrap();
  println!("results: {:?}", result);
}
```