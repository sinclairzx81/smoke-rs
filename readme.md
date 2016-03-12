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
  * [Mux / Demux streams](#mux_demux_streams)

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

The following creates a task which resolves a Result&lt;i32, i32&gt; and executes it asynchronously. 

When running the task with .async(), the body of the task is executed within a thread, in which the
result is pushed into the methods closure. In addition, the .async() method returns a JoinHandle for
which the caller can use to join() the thread back to the caller.

```rust
mod smoke;
use smoke::async::Task;

fn main() {
    let task = Task::<i32, i32>::new(|| {
        println!("inside the task");
        Ok(123)
    });
    
    task.async(|result| {
       let number = result.unwrap();
       // ...
    }).join();
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
   ]).sync().unwrap(); // [30, 50, 70]
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

<a name='mux_demux_streams'></a>
## Mux / Demux streams

Multiple of the same type can be merged into a single stream. The following creates two 
distinct streams (numbers and words), merges them into a stream and reads.

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