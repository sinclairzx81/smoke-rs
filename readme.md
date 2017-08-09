# smoke-rs

#### A lightweight async Task and Stream library for Rust

```rust
fn hello() -> Task<&'static str> {
  Task::new(|sender| {
    sender.send("hello world!!")
  })
}
```

## Overview

Smoke is a lightweight Task and Stream library for Rust. The library aims to help simplify 
asynchronous, concurrent and parallel programming in Rust by providing familiar primitives 
seen in languages like C# and JavaScript. 

Smoke's primary motive is to strike a good balance between Rust's thread safe semantics and the
ease of use of Tasks.

* [Task&lt;T&gt;](#task)
  * [Create Task](#creating_tasks)
  * [Run Sync](#run_sync)
  * [Run Async](#run_async)
  * [Run Parallel](#run_parallel)
  * [Scheduling](#scheduling)
* [Stream&lt;T&gt;](#stream)
  * [Output Streams](#output_streams)
  * [Input Streams](#input_streams)
  * [Merging](#merging_streams)
  * [Operators](#stream_operators)

<a name='task'></a>
## Task&lt;T&gt;

A Task encapsulates an asynchronous operation. 

<a name='creating_tasks'></a>
### Create Task

The following will create a Task&lt;i32&gt;. The value for this task is 
given by the call to sender.send() function.

```rust
use smoke::async::Task;

fn main() {
    let task = Task::new(|sender| {
        sender.send(123)
    });
}
```

<a name='run_sync'></a>
### Run Sync

To run a task synchronously, use the .wait() function. The .wait() 
function will block the current thread and return a Result&lt;T, RecvError&gt; 
when it can.

The following synchronously waits on a Task.

```rust
use smoke::async::Task;

fn main() {
    let task = Task::new(|sender| {
        sender.send("hello from task")
    });
    
    // blocking..
    println!("{}", task.wait().unwrap());
}
```

<a name='run_async'></a>
### Run Async

A task can be run asynchronously with the .async() function. The .async() 
function will pass a Result&lt;T, RecvError&gt; into the closure provided
and return a wait handle to the caller. The caller can use the handle to
synchronize the result back to the calling thread.

The following runs a Task asynchronously.

```rust
use smoke::async::Task;

fn main() {
    let task = Task::new(|sender| {
        sender.send("hello from task")
    });
    
    let handle = task.async(|result| {
      println!("{}", result.unwrap());
    });
    
    // sometime later...
    
    handle.wait();
}
```

<a name='run_parallel'></a>
### Run Parallel

Many tasks can be run in parallel by with the .all() function. This function accepts a vector
of type Vec&lt;Task&lt;T&gt;&gt; and gives back a new task of type Task&lt;Vec&lt;T&gt;&gt;.

The following demonstrates running tasks in parallel.

```rust
use smoke::async::Task;

fn compute() -> Task<i32> {
  Task::new(move |sender| {
    // emulate compute...
    Task::delay(1000).wait();
    sender.send(10)
  })
}

fn main() {
   // allocate 3 threads to process
   // these tasks.
   let result = Task::all(3, vec![
     compute(),
     compute(),
     compute()
   ]).wait().unwrap();
}
```

<a name="scheduling"></a>
### Scheduling

For convenience, tasks manage scheduling on behalf of the caller, however, in some
scenarios, it maybe desirable to have more control over task scheduling behavior.

Smoke provides 3 built-in scheduler types users can use to schedule tasks manually. 

These include:
* SyncScheduler - Tasks scheduled here will be executed in the current thread.
* ThreadScheduler - Tasks scheduled here will be executed in their own thread.
* ThreadPoolScheduler - Tasks executed here will be executed on a bounded threadpool.

```rust
use smoke::async::Task;
use smoke::async:: {
  SyncScheduler,
  ThreadScheduler,
  ThreadPoolScheduler
};

fn hello() -> Task<&'static str> {
  Task::delay(1).map(|_| "hello")
}

fn main() {
   // runs the task synchronously.
   let scheduler = SyncScheduler;
   let result    = hello().schedule(scheduler).wait();
   
   // creates a new thread for each task run.
   let scheduler = ThreadScheduler;
   let result    = hello().schedule(scheduler).wait();
      
   // schedules the task to be run on a 
   // threadpool, in this example, there
   // are 8 available threads.
   let scheduler = ThreadPoolScheduler::new(8);
   let result    = hello().schedule(scheduler).wait();   
}
```

<a name='stream'></a>
## Stream&lt;T&gt;

Stream&lt;T&gt; provides a simple abstraction to read and write streams of values asynchronously. 
Internally, Stream&lt;T&gt; is built over Rust mpsc channels, and manages the threading internals.

<a name='output_streams'></a>
### Output Streams

The following creates a function that creates a output stream of integer values. 
The main() function iterates values on the stream by calling .read(). Output 
streams are streams intended to be read externally to the stream.

```rust
use smoke::async::Stream;

fn numbers() -> Stream<i32> {
  let stream = Stream::output(|sender| {
      sender.send(1).unwrap();
      sender.send(2).unwrap();
      sender.send(3).unwrap();
      sender.send(4) // return mpsc 
  });  
}

fn main() {
  let stream = numbers();
  for n in stream.read() {
    // n = 1, 2, 3, 4
  }
}
```

<a name='input_streams'></a>
### Input Streams

Input streams are the direct inverse of a output stream. When creating
a input stream, the caller is returned a sender in which to send values. 
Values sent from the caller are received on the input streams receiver.

```rust
use smoke::async::{Stream, StreamSender};

fn numbers() -> StreamSender<i32> {
  let stream = Stream::input(|receiver| {
    for n in receiver {
      // n = 1, 2, 3, 4
    }
  });  
}

fn main() {
  let sender = numbers();
  sender.send(1).unwrap();
  sender.send(2).unwrap();
  sender.send(3).unwrap();
  sender.send(4).unwrap();
}
```

<a name='merging_streams'></a>
### Merging Streams

Output streams of the same type can be merged into a single stream with the .merge() function.

```rust
use smoke::async::Stream;

#[derive(Debug)]
enum Item {
  Number(i32),
  Word(&'static str)
}

fn numbers() -> Stream<Item> {
  Stream::output(|sender| {
    try! (sender.send(Item::Number(1)) );
    try! (sender.send(Item::Number(2)) );
    try! (sender.send(Item::Number(3)) );
    try! (sender.send(Item::Number(4)) );
    Ok(())
  }) 
}

fn words() -> Stream<Item> {
  Stream::output(|sender| {
      "the quick brown fox jumps over the lazy dog"
        .split(" ")
        .map(|n| sender.send(Item::Word(n)))
        .last()
        .unwrap()
  }) 
}

fn main() {
  let streams = vec![numbers(), words()];
  let merged  = Stream::merge(streams);
  for item in stream.read() {
    match item {
      Item::Word(word)     => println!("word: {:?}", word),
      Item::Number(number) => println!("number: {:?}", number)
    }
  }
}
```

<a name='stream_operators'></a>
### Stream Operators

Streams support .filter(), .map() and .fold() operators for stream composition. These operators are 
are deferred, and executed only when the stream is read().

```rust
use smoke::async::Stream;

fn numbers() -> Stream<i32> {
  Stream::output(|sender| {
    try! (sender.send(1) );
    try! (sender.send(2) );
    try! (sender.send(3) );
    sender.send(4)
  }) 
}

use std::fmt::Debug;
fn read<T: Debug + Send + 'static>(stream: Stream<T>) {
    for n in stream.read(0) {
      println!("{:?}", n);
    }
}

fn main() {
  // filter
  read(numbers().filter(|n| n % 2 == 0));
  // map
  read(numbers().map(|n| format!("num: {}", n)));
  // fold
  println!("{}", 
    numbers().fold(0, |p, c| p + c)
             .wait()
             .unwrap());
  // everything
  println!("{}", 
    numbers().filter(|n| n % 2 == 0)
             .map(|n| format!("{}", n))
             .fold(String::new(), |p, c| 
                    format!("{} {} and", p, c))
             .wait()
             .unwrap());
}
```
