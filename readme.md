#smoke-rs

Concurrency primitives for the Rust programming language.

## overview

This library provides task / stream primitives to help orchestrate concurrency in Rust.  

* [Task&lt;T&gt;](#task)
  * [Creating tasks](#creating_tasks)
  * [Run task synchronously](#run_task_synchronously)
  * [Run task asynchronously](#run_task_asynchronously)
  * [Run tasks in parallel](#run_tasks_in_parallel)
  * [Composing Tasks](#composing_tasks)
* [Stream&lt;T&gt;](#stream)
  * [Creating Streams](#creating_streams)
  * [Reading Streams](#reading_streams)
  * [Combining Streams](#combining_streams)
  * [Composing Streams](#composing_streams)
* [Examples](#examples)
  * [Processing files](#processing_files)

<a name='task'></a>
## Task&lt;T&gt;

A Task encapsulates a single asynchronous operation. 

<a name='creating_tasks'></a>
### Creating Tasks

The following will create a Task&lt;i32&gt;. The body of this task will not execute
until a caller calls either calls .sync() or .async() to obtain the result.

```rust
mod smoke;
use smoke::async::Task;

fn main() {
    let task = Task::new(|sender| {
        println!("inside the task");
        sender.send(123)
    });
}
```

<a name='run_task_synchronously'></a>
### Run Tasks synchronously

Tasks can be run synchronously with the .sync() function. The .sync() 
function returns a Result&lt;T, RecvError&gt; containing the result.

```rust
mod smoke;
use smoke::async::Task;

fn main() {
    let task = Task::new(|sender| {
        println!("inside the task");
        sender.send(123)
    });
    
    println!("{}", task.sync().unwrap());
}
```

<a name='run_task_asynchronously'></a>
### Run Tasks asynchronously

Tasks can be run asynchronously with the .async() function. The .async() 
function returns a JoinHandle&lt;T&gt; for the caller to .join() at some
point in the future.

```rust
mod smoke;
use smoke::async::Task;

fn main() {
    let task = Task::new(|sender| {
        println!("inside the task");
        sender.send(123)
    });
    
    let handle = task.async();
    // do other work...
    println!("{}", handle.join().unwrap());
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

fn add(a: i32, b: i32) -> Task<i32> {
  Task::new(move |sender| {
    sender.send(a + b)
  })
}

fn main() {
   let result = Task::all(vec![
     add(10, 20),
     add(20, 30),
     add(30, 40)
   ]).sync();
   
   // [30, 50, 70]
   println!("{:?}", result.unwrap()); 
}
```
<a name='composing_tasks'></a>
### Composing Tasks

Tasks can be composed into larger computations with the 
.then() function. The .then() function is similar to a .map() function
in the regard it allows tasks to be mapped into other tasks. The 
result is a new task that callers can use to execute the computation.

```rust
mod smoke;
use smoke::async::Task;

fn add(a: i32, b: i32) -> Task<i32> {
  Task::new(move |sender| {
    sender.send(a + b)
  })
}

fn main() {
   let value =  add(1, 2)
                  .then(|n| add(n, n+1))
                  .then(|n| add(n, n+1))
                  .then(|n| add(n, n+1))
                  .then(|n| add(n, n+1))
                  .sync();
                              
   // 63
   println!("{:?}", value.unwrap());
}
```

<a name='stream'></a>
## Stream&lt;T&gt;

Stream&lt;T&gt; provides a means to generate async sequences from 
which a caller may listen to. Internally, Stream&lt;T&gt; abstracts 
mpsc channels and provides some composability functions to aid in data 
flow.

<a name='creating_streams'></a>
## Creating Streams

The following creates a simple sequence of numbers. The streams closure 
expects a Result&lt;(), SendError&gt; result which can be obtain from the sender.
The stream ends once this closure finishes.

```rust
mod smoke;
use smoke::async::Stream;

fn main() {
  // a stream of numbers, 1 - 4.
  let stream = Stream::new(|sender| {
      try! (sender.send(1) );
      try! (sender.send(2) );
      try! (sender.send(3) );
      sender.send(4) // ok
  });
}
```

<a name='reading_streams'></a>
## Reading Streams

A stream can be read with the .async() and .sync(bound) functions. Internally, the
.async() and .sync(bound) provide a abstraction over a mpsc channel() or sync_channel(bound) 
respectively.

For details see..
* [std::sync::mpsc::channel](#https://doc.rust-lang.org/std/sync/mpsc/fn.channel.html)
* [std::sync::mpsc::sync_channel](#https://doc.rust-lang.org/std/sync/mpsc/fn.sync_channel.html)

Calling .sync() or .async() on a stream returns a Receiver&lt;T&gt; a caller can use
to iterate elements. 

```rust
mod smoke;
use smoke::async::Stream;

// creates a stream of numbers.
fn numbers() -> Stream<i32> {
    Stream::new(|sender| {
      (0..100)
        .map(|n| sender.send(n))
        .last()
        .unwrap()
    })
}

fn main() {
  for n in numbers().async() {
      println!("recv: {}", n);
  }
}

```

<a name='combining_streams'></a>
## Combining streams

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
  // combine ...
  let stream = Stream::combine(vec![
    numbers(),
    words()
  ]);
  // read ...
  for item in stream.async() {
    match item {
      Item::Word(word)     => println!("word: {:?}", word),
      Item::Number(number) => println!("number: {:?}", number)
    }
  }
}
```

<a name='composing_streams'></a>
## Composing Streams

Streams support composition in a similar fashion to Tasks. Callers
can use .map() .filter() and .fold() to map and modify the stream 
prior to calling .sync() or .async() to read the stream.

```rust
mod smoke;
use smoke::async::Stream;

fn numbers() -> Stream<i32> {
  Stream::new(|sender| {
    try! (sender.send(1) );
    try! (sender.send(2) );
    try! (sender.send(3) );
    sender.send(4)
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

#[derive(Debug)]
enum Item {
  Number(i32),
  Word(&'static str)
}

fn main() {
  // map streams ...
  let stream = Stream::combine(vec![
    numbers().map(|n| Item::Number(n)),
    words().map  (|n| Item::Word(n))
  ]);
  
  // read ...
  for item in stream.async() {
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

//------------------------------------------------
// filestream(): streams the bytes in this file.
//------------------------------------------------
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
// scan(): scan and count bytes..
//--------------------------------------
fn scan(filename: &'static str) -> Task<usize> {
  Task::new(move |sender| {
    println!("scan: {}", filename);
    let mut count = 0;
    for (size, _) in filestream(filename).sync(0) {
      count = count + size;
    } sender.send(count)
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