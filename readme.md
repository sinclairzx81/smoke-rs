#smoke-rs

A small experiment testing some common async primitives with rust. 

```rust

mod smoke;
use smoke::async::Task;
use std::thread;

fn ok() -> Task<i32, String> {
	Task::new(|task| {
		thread::spawn(move || {
			thread::sleep_ms(1000);
			task.call(Ok(1));
		});
	})
}

fn err() -> Task<i32, String> {
	Task::new(|task| {
		thread::spawn(move || {
			thread::sleep_ms(1000);
			task.call(Err("oh no".to_string()));
		});
	})
}

fn main() {
	// run task sync..
	let result = ok().sync();
	assert!(result.unwrap() == 1);
	
	// run task async..
	ok().then(|result| {
		assert!(result.unwrap() == 1);
	}).async();
	
	// run tasks in series
	let result = Task::series(vec![ok(), ok(), ok()]).sync();
	match result {
		Ok(vec) => {
			for element in vec {
				assert!(element == 1);	
			}
		},
		Err(_)  => panic!()
	}
	
	// run tasks in parallel
	let result = Task::parallel(vec![ok(), ok(), ok()]).sync();
	match result {
		Ok(vec)   => {
			for element in vec {
				assert!(element == 1);
			}	
		},
		Err(_)  => panic!()
	}
	
	// run tasks in parallel, expect err()
	let result = Task::parallel(vec![ok(), err(), ok()]).sync();
	match result {
		Ok(_)      => panic!(),
		Err(error) => {}
	}
}
```


  