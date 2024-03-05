use std::fs;
// use std::io::prelude::*;
// use std::net::TcpListener; // This crate was replaced by async_std::net::TcpListener;
// use std::net::TcpStream;   // This crate was replaced by async_std::net::TcpStream;

use async_std::prelude::*;
use async_std::net::TcpListener;
// use async_std::net::TcpStream; // handle_connection doesn't actually require this. It requires
// any struct that implements async::std::io::Read, async_std::io::Write, and marker::Unpin

use async_std::io::{Read, Write}; 
use async_std::task::spawn;
use futures::stream::StreamExt;

use std::time::Duration;
use async_std::task;

#[async_std::main]
async fn main() {
    // Listen for incoming Tcp connections on localhost port 7878
    let listener = TcpListener::bind("127.0.0.1:7878").await.unwrap();
    listener
        .incoming()
        .for_each_concurrent(/*limit*/ None, |stream| async move {
            let stream = stream.unwrap();
            spawn(handle_connection(stream));
    })
    .await;
}

// async fn handle_connection(mut stream: TcpStream) {
// Changing the type signature to reflect line 8 comment allows us to pass a mock for testing.

   async fn handle_connection(mut stream: impl Read + Write + Unpin) { 
    // Read the first 1024 bytes of data from the stream
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).await.unwrap();

    let get = b"GET / HTTP/1.1\r\n";    // byte string literal
    let sleep = b"GET /sleep HTTP/1.1\r\n";

    // Respond with greetings or a 404,
    // depending on the data in the request
    let (status_line, filename) = if buffer.starts_with(get) {
        ("HTTP/1.1 200 OK\r\n\r\n", "hello.html")
    } else if buffer.starts_with(sleep) {
        task::sleep(Duration::from_secs(5)).await;
        ("HTTP/1.1 200 OK\r\n\r\n", "hello.html")
    } else {
        ("HTTP/1.1 404 NOT FOUND\r\n\r\n", "404.html")
    };
    let contents = fs::read_to_string(filename).unwrap();

    // Write response back to the stream,
    // and flush the stream to ensure the response is sent back to the client
    let response = format!("{status_line}{contents}");
    stream.write_all(response.as_bytes()).await.unwrap();
    stream.flush().await.unwrap();
}

//TODO: I feel like the test goes in a diff file due to use of super::*
// Adding async to the function declaration changes its return type from the unit type () to a 
// type that implements Future<Output = ()>


//Next, lets build a mock TcpStream that implements these traits. First, let's implement the Read
//trait, with one method, poll_read. Our mock TcpStream will contain some data that is copied into
//the read buffer, and we'll return Poll::Ready to signify that the read is complete.
// use super::*;
use futures::io::Error;
use futures::task::{Context, Poll};

use std::cmp::min;
use std::pin::Pin;

struct MockTcpStream {
    read_data: Vec<u8>,
    write_data: Vec<u8>,
}

impl Read for MockTcpStream {
    fn poll_read(
        self: Pin<&mut Self>,
        _: &mut Context,
        buf: &mut [u8],
    ) -> Poll<Result<usize, Error>> {
        let size: usize = min(self.read_data.len(), buf.len());
        buf[..size].copy_from_slice(&self.read_data[..size]);
        Poll::Ready(Ok(size))
    }
}

//Our implementation of write is very similar, although we'll need to write three methods:
//poll_wrtie, poll_flush, and poll_close. poll_write will copy any input data into the mock
//TcpStream, and return Poll::Ready when complete. No work needs to be done to flush or close
//the mock TcpStream, so pull_flush and poll_close can just return Poll::Ready.

impl Write for MockTcpStream {
    fn poll_write(
        mut self: Pin<&mut Self>,
        _: &mut Context,
        buf: &[u8],
    ) -> Poll<Result<usize, Error>> {
        self.write_data = Vec::from(buf);

        Poll::Ready(Ok(buf.len()))
    }

    fn poll_flush(self: Pin<&mut Self>, _: &mut Context) -> Poll<Result<(), Error>> {
        Poll::Ready(Ok(()))
    }

    fn poll_close(self: Pin<&mut Self>, _: &mut Context) -> Poll<Result<(), Error>> {
        Poll::Ready(Ok(()))
    }
}

// Lastly, our mock will need to implement Unpin, signifying that its location in memory can safely
// be moved.
impl Unpin for MockTcpStream {}



// Now we are ready to test the handle_connection function. After setting up the MockTcpStream
// containing some initial data, we can run handle_connection using the 
// attribute #[async_std::test], similarly to how we sued #[async_std::main]. To ensure that
// handle_connection works as intended, we'll check that the correct data was written to the 
// MockTcpStream based on its initial contents.

 // use std::fs;

#[async_std::test]
async fn test_handle_connection() {
    let input_bytes = b"GET / HTTP/1.1\r\n";
    let mut contents = vec![0u8; 1024];
    contents[..input_bytes.len()].clone_from_slice(input_bytes);
    let mut stream = MockTcpStream {
        read_data: contents,
        write_data: Vec::new(),
    };

    handle_connection(&mut stream).await;

    let expected_contents = fs::read_to_string("hello.html").unwrap();
    let expected_response = format!("HTTP/1.1 200 OK\r\n\r\n{}", expected_contents);
    assert!(stream.write_data.starts_with(expected_response.as_bytes()));
}





