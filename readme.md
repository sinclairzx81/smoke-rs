#smoke-rs

Task based asynchronous programming for Rust.

```rust
use smoke::async::Task;

fn hello() -> Task<&'static str> {
  Task::new(|sender| {
    sender.send("hello world!!")
  })
}

fn main() {
  hello().async(|result| {
    println!("{:?}", result);
  })
}
```

## overview

This library provides task / stream primitives to orchestrate concurrency in Rust. 

* [Task&lt;T&gt;](#task)
  * [Creating Tasks](#creating_tasks)
  * [Run Tasks Synchronously](#run_tasks_synchronously)
  * [Run Tasks Asynchronously](#run_tasks_asynchronously)
  * [Run Tasks in Parallel](#run_tasks_in_parallel)
  * [Composing Tasks](#composing_tasks)
  * [Scheduling Tasks](#scheduling_tasks)
* [Stream&lt;T&gt;](#stream)
  * [Creating Streams](#creating_streams)
  * [Reading Streams](#reading_streams)
  * [Combining Streams](#combining_streams)
  * [Composing Streams](#composing_streams)

<a name='task'></a>
## Task&lt;T&gt;

A Task encapsulates a asynchronous operation. 

<a name='creating_tasks'></a>
### Creating Tasks

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

<a name='run_tasks_synchronously'></a>
### Run Tasks Synchronously

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
    // blocking
    println!("{}", task.sync().unwrap());
}
```

<a name='run_tasks_asynchronously'></a>
### Run Tasks Asynchronously

Tasks can be run asynchronously with the .async() function. The .async() 
function will pass a Result&lt;T, RecvError&gt; into the closure provided
and return a wait handle to the caller. 

The following runs a Task asynchronously.

```rust
use smoke::async::Task;

fn main() {
    let task = Task::new(|sender| {
        sender.send("hello from task")
    });
    
    task.async(|result| {
      // inside thread ..
      println!("{}", result.unwrap());
    });
    // program ends...
}
```
The .async() function can returns a wait handle the caller can use to
wait for the async operation to complete. In addition, it may also return
results to the calling thread..

```rust
use smoke::async::Task;

fn main() {
    let task = Task::new(|sender| {
        sender.send("hello from task")
    });
    
    let result = task.async(|result| {
      println!("{}", result.unwrap());
      "hello from closure"
    }).wait(); 
    
    println!("{}", result.unwrap());
}
```
<a name='run_tasks_in_parallel'></a>
### Run Tasks in Parallel

Tasks can be run in parallel by with the .all() function. The all() function accepts a vector
of type Vec&lt;Task&lt;T&gt;&gt; and gives back a new task of type Task&lt;Vec&lt;T&gt;&gt; which
contains the results for each task given.

The following demonstrates running tasks in parallel.

```rust
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

Tasks can be composed with the .then() function. Tasks chained
with the .then() function will result in sequential execution of the 
tasks in the chain. 

The .then() function accepts a task of type Task&lt;T&gt; and returns a new task
of type Task&lt;U&gt; which allows a caller to optionally map the 
original task to a new type. 

The following demostrates some basic composition.


```rust
use smoke::async::Task;

// simple add function.
fn add(a: i32, b: i32) -> Task<i32> {
  Task::new(move |sender| {
    sender.send(a + b)
  })
}
// simple format function.
use std::fmt::Debug;
fn format<T>(t: T) -> Task<String> where
   T:Debug + Send + 'static {
   Task::new(move |sender| {
      let n = format!("result is: {:?}", t);
      sender.send(n)
   })
}
fn main() {
   
   // add two numbers
   let result = add(1, 2).wait();
   println!("{:?}", result.unwrap()); 
   
   // wait 1 second.
   // add two numbers.
   let result = Task::delay(1000)
                .then(|_| add(10, 20))
                .wait();
                
   println!("{:?}", result.unwrap()); 
              
   // wait 1 second.
   // add two numbers.
   // add 3.
   // format.
   let result = Task::delay(1000)
                .then(|_|   add(100, 200))
                .then(|res| add(res.unwrap(), 3))
                .then(|res| format(res.unwrap()))
                .wait();
   
   println!("{:?}", result.unwrap());
}
```

<a name='stream'></a>
## Stream&lt;T&gt;

Stream&lt;T&gt; provides a means to generate async sequences from 
which a caller may listen to. Internally, Stream&lt;T&gt; abstracts 
mpsc channels and provides some composability functions to aid in data 
flow.

<a name='creating_streams'></a>
### Creating Streams

The following creates a simple sequence of numbers. The streams closure 
expects a Result&lt;(), SendError&gt; result which can be obtain from the sender.
The stream ends once this closure finishes.

```rust
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
### Reading Streams

A stream can be read with the .async() and .sync(bound) functions. Internally, the
.async() and .sync(bound) provide a abstraction over a mpsc channel() or sync_channel(bound) 
respectively.

For details see..
* [std::sync::mpsc::channel](#https://doc.rust-lang.org/std/sync/mpsc/fn.channel.html)
* [std::sync::mpsc::sync_channel](#https://doc.rust-lang.org/std/sync/mpsc/fn.sync_channel.html)

Calling .sync() or .async() on a stream returns a Receiver&lt;T&gt; a caller can use
to iterate elements. 

```rust
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
### Combining Streams

Multiple streams of the same type can be merged into a single stream. The following creates two 
distinct streams (numbers and words), merges them into a single stream and reads.

```rust
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
### Composing Streams

Streams support composition in a similar fashion to Tasks. Callers
can use .map() .filter() and .fold() to map and modify the stream 
prior to calling .sync() or .async() to read the stream.

```rust
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