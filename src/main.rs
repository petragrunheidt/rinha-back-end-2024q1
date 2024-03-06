use std::net::{TcpListener, TcpStream};
use std::io::Read;
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
  let start_index = request_str.find('{');

  if let Some(start) = start_index {
      let mut depth = 1;
      let mut end_index = None;

      for (index, char) in request_str[start + 1..].char_indices() {
          match char {
              '{' => depth += 1,
              '}' => {
                  depth -= 1;
                  if depth == 0 {
                      end_index = Some(start + index + 1);
                      break;
                  }
              }
              _ => {}
          }
      }

      if let Some(end) = end_index {
          return Some(request_str[start..=end].to_string());
      }
  }
  None
}

// curl -X POST -H "Content-Type: application/json" -d '{ "valor": 1000, "tipo": "c", "descricao": "descricao" }' localhost:9999/clientes/1/transacoes