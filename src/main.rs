use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};

fn main() {
    let listener = TcpListener::bind("0.0.0.0:8080").expect("Failed to bind");

    println!("Server listening on port 8080");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                std::thread::spawn(move || {
                    handle_connection(stream);
                });
            }
            Err(e) => eprintln!("Error accepting connection: {}", e),
        }
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).expect("Failed to read request");

    println!("Received request: {}", String::from_utf8_lossy(&buffer));

    let request_str = String::from_utf8_lossy(&buffer);
    let http_method = request_str.split_whitespace().next();
    let path = request_str.split_whitespace().nth(1).unwrap_or("/");

    match (http_method, path) {
      (Some("GET"), "/hello") => {
          handle_hello_request(&mut stream);
      }
      (Some("POST"), "/world") => {
          handle_world_request(&mut stream);
      }
      _ => {
          let response = "HTTP/1.1 405";
          stream.write_all(response.as_bytes()).expect("Failed to write response");
          stream.flush().expect("Failed to flush");
      }
    }
  }

fn handle_hello_request(stream: &mut TcpStream) {
  let response = "HTTP/1.1 200\r\nContent-Type: text/plain\r\n\r\nHihi, World!";
  stream.write_all(response.as_bytes()).expect("Failed to write response");
  stream.flush().expect("Failed to flush");
}

fn handle_world_request(stream: &mut TcpStream) {
  let response = "HTTP/1.1 200\r\nContent-Type: text/plain\r\n\r\nByebye, World!";
  stream.write_all(response.as_bytes()).expect("Failed to write response");
  stream.flush().expect("Failed to flush");
}