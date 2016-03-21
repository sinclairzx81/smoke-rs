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
use std::io::{Read, BufRead, BufReader};

use super::async::Task;
use super::async::Stream;

/// Asynchronous extensions on the Read trait.
pub trait ReadAsync {
  
  /// Asynchronously reads all bytes until EOF.
  ///
  /// #Example
  /// ```
  /// use smoke::io::ReadAsync;
  /// 
  /// let read = std::io::empty();
  /// 
  /// let task = read.read_to_end_task().async(|result| {
  ///   // bytes available here.
  /// });
  /// ```
  fn read_to_end_task(self: Self)    -> Task<Vec<u8>>;
  
  /// Asynchronously reads all bytes until EOF. 
  ///
  /// #Example
  /// ```
  /// use smoke::io::ReadAsync;
  /// 
  /// let read = std::io::empty();
  /// 
  /// let task = read.read_to_string_task().async(|result| {
  ///   // bytes available here.
  /// });
  /// ``` 
  fn read_to_string_task(self: Self) -> Task<String>;
  
  /// Streams bytes until EOF.
  ///
  /// #Example
  /// ```
  /// use smoke::io::ReadAsync;
  /// 
  /// let read = std::io::empty();
  /// 
  /// // 16k
  /// for bytes in read.read_stream(16384).read(0) {
  ///   println!("{}", bytes.len());
  /// }
  /// ```
  fn read_stream(self: Self, size: usize) -> Stream<Vec<u8>>;
  
  /// Stream lines until EOF.
  ///
  /// #Example
  /// ```
  /// use smoke::io::ReadAsync;
  ///
  /// let read = std::io::empty();
  /// 
  /// // 16k
  /// for bytes in read.read_line_stream().read(0) {
  ///   println!("{}", bytes.len());
  /// }
  /// ```  
  fn read_line_stream(self: Self) -> Stream<String>;
}

impl<R: Read + Send + 'static> ReadAsync for R {
  
  /// Asynchronously reads all bytes until EOF. 
  fn read_to_end_task(self: Self) -> Task<Vec<u8>> {
    let reader = Mutex::new(self);
    Task::new(move |sender| {
      let mut reader = reader.lock().unwrap();
      let mut buf    = Vec::new();
      reader.read_to_end(&mut buf).unwrap();
      sender.send(buf)
    })
  }
  
  /// Asynchronously reads all bytes until EOF. 
  fn read_to_string_task(self: Self) -> Task<String> {
    let reader = Mutex::new(self);
    Task::new(move |sender| {
      let mut reader = reader.lock().unwrap();
      let mut buf    = String::new();
      reader.read_to_string(&mut buf).unwrap();
      sender.send(buf)
    })
  }
  
  /// Stream bytes until EOF.
  fn read_stream(self: Self, bufsize: usize) -> Stream<Vec<u8>> {
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
  fn read_line_stream(self: Self) -> Stream<String> {
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