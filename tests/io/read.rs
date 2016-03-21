use smoke::io::Read;
use std::io::empty;

#[test]
fn to_stream() {
  let read = empty();
  let stream = read.to_stream(16384);
  for _ in stream.read(0) {}
}

#[test]
fn to_line_stream() {
  let read = empty();
  let stream = read.to_line_stream();
  for _ in stream.read(0) {}
}