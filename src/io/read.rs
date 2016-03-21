/*--------------------------------------------------------------------------
 smoke-rs

 The MIT License (MIT)

 Copyright (c) 2016 Haydn Paterson (sinclair) <haydn.developer@gmail.com>

 Permission is hereby granted, free of charge, to any person obtaining a copy
 of this software and associated documentation files (the "Software"), to deal
 in the Software without restriction, including without limitation the rights
 to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 copies of the Software, and to permit persons to whom the Software is
 furnished to do so, subject to the following conditions:

 The above copyright notice and this permission notice shall be included in
 all copies or substantial portions of the Software.

 THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
 THE SOFTWARE.
---------------------------------------------------------------------------*/

use std::sync::Mutex;
use std::io::{Read as StdRead, BufRead, BufReader};
use super::super::async::Stream;

/// Adds asynchronous operations over the std::io::Read trait.
pub trait Read : StdRead {
  
  /// Streams bytes until EOF.
  ///
  /// #Example
  /// ```
  /// use smoke::io::Read;
  /// 
  /// let read = std::io::empty();
  /// 
  /// // to stream with 16k chunks.
  /// for bytes in read.to_stream(16384).read(0) {
  ///   println!("{}", bytes.len());
  /// }
  /// ```
  fn to_stream(self: Self, size: usize) -> Stream<Vec<u8>>;
  
  /// Stream lines until EOF.
  ///
  /// #Example
  /// ```
  /// use smoke::io::Read;
  /// 
  /// let read = std::io::empty();
  /// 
  /// for line in read.to_line_stream().read(0) {
  ///   println!("{}", line);
  /// }
  /// ```
  fn to_line_stream(self: Self) -> Stream<String>;
}

impl<R: StdRead + Send + 'static> Read for R {
  
  /// Stream bytes until EOF.
  fn to_stream(self: Self, bufsize: usize) -> Stream<Vec<u8>> {
      let reader = Mutex::new(self);
      Stream::new(move |sender| {
        let mut reader = reader.lock().unwrap();
        let mut buf    = vec![0; bufsize];
        loop {
          let read = reader.read(&mut buf).unwrap();
          if read > 0 {
            try!(sender.send(buf[0..read].to_vec()));
          } else {
            break;
          }
        } Ok(())    
      })
  }
  
  /// Stream lines until EOF.
  fn to_line_stream(self: Self) -> Stream<String> {
      let reader = Mutex::new(Some(self));
      Stream::new(move |sender| {
        let mut reader = reader.lock().unwrap();
        let reader     = reader.take();
        let mut reader = BufReader::new(reader.unwrap());
        let mut buf    = String::new();
        while reader.read_line(&mut buf).unwrap() > 0 {
            try!(sender.send(buf.clone()));
            buf.clear();
        } Ok(())    
      })
  }  
}