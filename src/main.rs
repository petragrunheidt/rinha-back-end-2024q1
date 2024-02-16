use std::net::{TcpListener, TcpStream};
use std::io::Read;

mod pg_pool;
use pg_pool::create_pg_pool;

mod controllers;
use controllers::{handle_extract_request, handle_transaction_request};
fn main() {
    // 127.0.0.1 0.0.0.0
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
    let pg_conn = create_pg_pool();

    let request_str = String::from_utf8_lossy(&buffer);
    let http_method = request_str.split_whitespace().next();
    let path = request_str.split_whitespace().nth(1).unwrap_or("/");
    let split_path: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

    if split_path[0] == "clientes" {
      let client_id = split_path[1];
      let sub_path = split_path[2];

      match (http_method, sub_path) {
        (Some("GET"), "extrato") => {
            handle_extract_request(&mut stream, pg_conn, client_id);
        }
        (Some("POST"), "transacoes") => {
            handle_transaction_request(&mut stream, pg_conn, client_id);
        }
        _ => {}
      }
    }
  }

