#smoke-rs

#### A lightweight async Task and Stream library for Rust

```rust
fn hello() -> Task<&'static str> {
  Task::new(|sender| {
    sender.send("hello world!!")
  })
}
```

## overview

Smoke is a lightweight Task and Stream library for Rust. The library aims to help simplify 
asynchronous, concurrent and parallel programming in Rust by providing familar primitives 
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
  * [Create Stream](#creating_streams)
  * [Reading](#reading_streams)
  * [Merging](#merging_streams)
  * [Operators](#stream_operators)

<a name='task'></a>
## Task&lt;T&gt;

A Task encapsulates a asynchronous operation. 

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
    println!("{}", task.sync().unwrap());
}
```

<a name='run_async'></a>
### Run Async

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
      println!("{}", result.unwrap());
    });
    // program ends...
}
```
The .async() function returns a handle which the caller can use
to wait for completion. Results of the async closure can be obtained
on the handle. 

```rust
use smoke::async::Task;

fn main() {
    let task = Task::new(|sender| {
        sender.send("hello from task")
    });
    
    let handle = task.async(|result| {
      println!("{}", result.unwrap());
      "hello from closure"
    });
    
    // sometime later...
    println!("{}", handle.wait().unwrap());
}
```
<a name='run_parallel'></a>
### Run Parallel

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
   // allocate 3 threads to process
   // these tasks.
   let result = Task::all(3, vec![
     add(10, 20),
     add(20, 30),
     add(30, 40)
   ]).sync();
   
   // [30, 50, 70]
   println!("{:?}", result.unwrap()); 
}
```

<a name="scheduling"></a>
### Scheduling

For convenience, Tasks are run can be waited on and run asynchronously without
needing to be concerned about how tasks are run. Internally, tasks are scheduled
to run on a variety of schedulers managed by the Task. However, in some instances, 
it may be desirable to override the default scheduling behavior. 

Smoke provides 3 built in schedulers users can use to schedule tasks manually. 

These include:
* SyncScheduler - Tasks scheduled here are executed in the current thread.
* ThreadScheduler - Tasks scheduled here are executed in there own thread.
* ThreadPoolScheduler - Tasks executed here are executed on a bounded threadpool.

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

Stream&lt;T&gt; provides a means to generate async sequences.

<a name='creating_streams'></a>
### Create Stream

The following creates a stream of numbers. 

```rust
use smoke::async::Stream;

fn main() {
  // a stream of numbers, 1 - 4.
  let stream = Stream::new(|sender| {
      try! ( sender.send(1) );
      try! ( sender.send(2) );
      try! ( sender.send(3) );
      sender.send(4)
  });
}
```

<a name='reading_streams'></a>
### Reading Streams

Use the .read(n) function to begin reading from a stream. The
read function will internally spawn a new thread for the body
of the stream closure and return to the caller a mpsc Receiver.

The .read(n) function takes a bound. The bound correlates to a
mpsc sync_channel bound. Setting a bound of 0 will cause the
sending thread to block on each iteration, setting a higher
bound allows for asynchronous buffering.

```rust
use smoke::async::Stream;

fn main() {
  // a stream of numbers, 1 - 4.
  let stream = Stream::new(|sender| {
      try! ( sender.send(1) );
      try! ( sender.send(2) );
      try! ( sender.send(3) );
      sender.send(4)
  }); 
  
  for n in stream.read(0) {
      println!("recv: {}", n);
  }
}

```

<a name='merging_streams'></a>
### Merging Streams

Streams of the same type can be merged with the .merge() function.

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
  let streams = vec![numbers(), words()];
  let merged  = Stream::merge(streams);
  for item in stream.read(0) {
    match item {
      Item::Word(word)     => println!("word: {:?}", word),
      Item::Number(number) => println!("number: {:?}", number)
    }
  }
}
```

<a name='stream_operators'></a>
### Stream Operators

Streams support some operators to transform a stream prior to reading a stream. This allows
for some stream composition. Streams currently support .filter(), .map() and .fold().

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