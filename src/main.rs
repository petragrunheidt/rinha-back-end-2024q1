use std::net::{TcpListener, TcpStream};
use std::io::Read;
use regex::Regex;
mod pg_pool;
use pg_pool::create_pg_pool;
mod extract_controller;
use extract_controller::handle_extract_request;
mod transaction_controller;
use transaction_controller::handle_transaction_request;

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
  let mut request_body = String::new();
  
  stream.read(&mut buffer).expect("Failed to read request");
  let request_str = String::from_utf8_lossy(&buffer);
  println!("Received request: {}", request_str);

  let pg_conn = create_pg_pool();

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
        if let Some(json_str) = extract_json_from_request(&request_str) {
          request_body.push_str(&json_str);
        } else {
            eprintln!("No JSON object found in the request body");
        }      
        handle_transaction_request(&mut stream, pg_conn, client_id, &request_body);
      }
      _ => {}
    }
  }
}

fn extract_json_from_request(request_str: &str) -> Option<String> {
  // Define a regular expression pattern to match JSON objects
  let json_regex = Regex::new(r"\{.*\}").expect("Failed to create regex");

  // Search for the JSON object in the request body
  if let Some(mat) = json_regex.find(request_str) {
      // Extract the matched substring (which contains the JSON data)
      Some(mat.as_str().to_string())
  } else {
      // Return None if no JSON object is found
      None
  }
}
