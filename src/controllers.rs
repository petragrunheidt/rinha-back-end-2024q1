use std::net::TcpStream;
use std::io::Write;
use std::thread;
use std::sync::{Arc, Mutex};
use r2d2_postgres::{postgres::NoTls, PostgresConnectionManager};
use r2d2::Pool;

pub fn handle_extract_request(stream: &mut TcpStream, pg_pool: Pool<PostgresConnectionManager<NoTls>>, client_id: &str) {
    let pool = pg_pool.clone();
    let client_id_clone: String = client_id.to_string();

    let json_response = Arc::new(Mutex::new(String::new()));
    let json_response_clone = Arc::clone(&json_response);

    thread::spawn(move || {
        let mut client = pool.get().unwrap();
        
        let balance_query = format!("SELECT a.limit_amount, b.amount FROM accounts AS a JOIN balances AS b ON a.id = b.account_id WHERE a.id = {}", id);
        let balance_result = client.query_one(&balance_query, &[]);

        match balance_result {
            Ok(row) => {
              let limit_amount: i32 = row.get(0);
              let amount: i32 = row.get(1);
              let mut json_response = json_response_clone.lock().unwrap();
              *json_response = format!(
                r#"{{ "client_id": "{}", "limit_amount": {}, "amount": {} }}"#,
                client_id_clone, limit_amount, amount
              );
            },
            Err(e) => {
              eprintln!("Failed to retrieve balance for client {}: {}", client_id_clone, e);
              let mut json_response = json_response_clone.lock().unwrap();
              *json_response = format!(r#"{{ "error": "Failed to retrieve balance for client {}" }}"#, client_id_clone);
            }
        }
    }).join().expect("The thread being joined has panicked");

    let json_response = json_response.lock().unwrap().to_string();
    handle_response(stream, "200 OK", json_response);
}


pub fn handle_transaction_request(stream: &mut TcpStream, pg_pool: Pool<PostgresConnectionManager<NoTls>>, client_id: &str) {
  // let _pg_client = PG_CLIENT.lock().unwrap();

  let response = format!("HTTP/1.1 200\r\nContent-Type: text/plain\r\n\r\nHihi, {} - Transaction!", client_id);
  stream.write_all(response.as_bytes()).expect("Failed to write response");
  stream.flush().expect("Failed to flush");
}

fn handle_response(tcp_stream: &mut TcpStream, status: &str, content: String) {
  let response_http = format!("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}", content.len(), content);
  tcp_stream.write_all(response_http.as_bytes()).expect("Failed to write response");
  tcp_stream.flush().expect("Failed to flush");
}