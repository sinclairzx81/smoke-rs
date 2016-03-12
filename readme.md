#smoke-rs

Asynchronous primitives for the Rust programming language.

## overview

This library provides two async primitives to help orchestrate concurrent and parallel programming
scenarios in the Rust language.

* [Task&lt;T, E&gt;](#task)
  * [creating tasks](#creating_tasks)
  * [run task synchronously](#run_task_synchronously)
  * [run task asynchronously](#run_task_asynchronously)
  * [run tasks in parallel](#run_tasks_in_parallel)
* [Stream&lt;T&gt;](#stream)
	* [creating streams](#creating_streams)
	* [merge streams](#merge_streams)
	* [filtering and mapping streams](#filtering_and_mapping_streams)

## Task&lt;T, E&gt;

A task encapsulates a single operation and provides a means to run that operation 
synchronously or asynchronously. A task achieves this by providing the caller a .sync() 
or .async() method which the caller can use to resolve a std::Result&lt;T, E&gt;.

### creating tasks

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

### run task synchronously

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

### run task asynchronously

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

### run tasks in parallel

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
## Stream&lt;T&gt;

Stream&lt;T&gt; provides a means to generate async sequences from 
which a caller may listen to. Internally, Stream&lt;T&gt; abstracts 
mpsc channels and provides some composability methods to aid in data flow.

## creating streams

The following creates a simple sequence of numbers. The caller
calls .recv() to start the receiving items from the stream. Internally
the stream is executed within its own thread.

```rust
mod smoke;
use smoke::async::Stream;
use std::thread;
use std::time::Duration;

// create a stream of numbers..
fn numbers() -> Stream<i32> {
  Stream::new(|sender| {
      (0..10).map(|n| {
        thread::sleep(Duration::from_secs(1));
        sender.send(n)
      }).last().unwrap()
  }) 
}

fn main() {
  for n in numbers().recv() {
      println!("{}", n);
  }
}
```

## merge streams

Streams of the same type can be merged. The following creates two distinct streams 
and merges them into one.

```rust

mod smoke;
use smoke::async::Stream;
use std::thread;
use std::time::Duration;

#[derive(Debug)]
enum Foo {
  Number(i32),
  Word(&'static str)
}

// a stream of numbers.
fn numbers() -> Stream<Foo> {
  Stream::new(|sender| {
      (0..10).map(|n| {
          thread::sleep(Duration::from_secs(1));
          sender.send(Foo::Number(n))
      }).last().unwrap()
  }) 
}

// a stream of words.
fn words() -> Stream<Foo> {
  Stream::new(|sender| {
      "the quick brown fox jumps over the lazy dog"
        .split(" ")
        .map(|n| {
          thread::sleep(Duration::from_secs(1));
          sender.send(Foo::Word(n))
      }).last().unwrap()
  }) 
}

fn main() {
  let stream = Stream::merge(vec![
      numbers(), 
      words()
      ]);
  
  for n in stream.recv() {
      println!("{:?}", n);
  }
}
```

## filtering and mapping streams.

Streams can be mapped and filtered.

```rust
mod smoke;
use smoke::async::Stream;

// create a stream of numbers..
fn numbers() -> Stream<i32> {
  Stream::new(|sender| {
      (0..10).map(|n| sender.send(n))
             .last()
             .unwrap()
  }) 
}

fn main() {
  for n in numbers().filter(|n| n % 2 == 0) // only even numbers
                    .map   (|n| n * 2)      // multiple them by 2.
                    .recv  () {
      println!("{}", n);
  }
}
```