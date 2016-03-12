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
  * [Map / Reduce streams](#map_reduce_streams)
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
    
    // run task synchronously. 
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
   // run in parallel.
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

The following creates a simple sequence of numbers and reads from it. It is 
important to note, the stream will only begin reading once the caller
calls .recv() on the stream. Until such time, the stream can be considered
in a pending state.

```rust
mod smoke;
use smoke::async::Stream;

fn numbers() -> Stream<i32> {
    Stream::new(|sender| {
      println!("inside stream");
      try! (sender.send(1) );
      try! (sender.send(2) );
      try! (sender.send(3) );
      try! (sender.send(4) );
      Ok(())
    })
}

fn main() {
  // create stream.
  let stream = numbers();
  
  // reading starts here.
  for n in stream.recv() {
      println!("{}", n);
  }
}
```

<a name='map_reduce_streams'></a>
## Map / Reduce streams

Streams support filter, map and reduce operators. These operators can be applied to
a stream before reading begins on the stream. 

```rust
mod smoke;
use smoke::async::Stream;

fn numbers() -> Stream<i32> {
  Stream::new(|sender| {
      try! (sender.send(1) );
      try! (sender.send(2) );
      try! (sender.send(3) );
      try! (sender.send(4) );
      Ok(())
  }) 
}

fn main() {
  let stream = numbers();
  
  let task = 
      stream.filter(|n| n % 2 == 0)   // Stream<T> -> Stream<T>
            .map   (|n| n * 2)        // Stream<T> -> Stream<U>
            .reduce(|p, c| p + c, 0); // Stream<U> -> Task<U>
              
   println!("{}", task.sync().unwrap());
}
```

<a name='mux_demux_streams'></a>
## Mux / Demux streams

Multiple of the same type can be merged into a single stream. The following creates two 
distinct streams (numbers and words), merges them into a single stream followed by seperating
them in the for loop.

```rust
mod smoke;
use smoke::async::Stream;

#[derive(Debug)]
enum Foo {
  Number(i32),
  Word(&'static str)
}

fn numbers() -> Stream<Foo> {
  Stream::new(|sender| {
    try! (sender.send(Foo::Number(1)) );
    try! (sender.send(Foo::Number(2)) );
    try! (sender.send(Foo::Number(3)) );
    try! (sender.send(Foo::Number(4)) );
    Ok(())
  }) 
}

fn words() -> Stream<Foo> {
  Stream::new(|sender| {
      "the quick brown fox jumps over the lazy dog"
        .split(" ")
        .map(|n| sender.send(Foo::Word(n)))
        .last()
        .unwrap()
  }) 
}

fn main() {
  // mux
  let stream = Stream::mux(vec![
      numbers(), 
      words()
      ]);
  
  // demux
  for foo in stream.recv() {
      match foo {
        Foo::Number(n) => 
          println!("number -> {}", n),
        Foo::Word(n) => 
          println!("word -> {}", n)
      }
  }
}
```