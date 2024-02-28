use std::net::TcpStream;
use std::io::Write;
use std::thread;
use std::sync::{Arc, Mutex};
use r2d2_postgres::{postgres::NoTls, PostgresConnectionManager};
use r2d2::Pool;
use serde::{Deserialize, Serialize};
use serde_json;

struct Response {
  status: String,
  body: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Transaction {
  valor: i32,
  tipo: String,
  descricao: String,
}

fn get_balance_query(id: &str) -> String {format!("SELECT a.limit_amount, b.amount FROM accounts AS a JOIN balances AS b ON a.id = b.account_id WHERE a.id = {}", id)}
fn update_balance_query(id: &str, amount: &i32, transaction_type: &str) -> String {
  match transaction_type {
      "c" => format!("UPDATE accounts SET limit_amount = limit_amount - {} WHERE id = {}", amount, id),
      "d" => format!("UPDATE balances SET amount = amount - {} WHERE id = {}", amount, id),
      _ => panic!("Invalid transaction type"),
  }
}
fn register_transaction_query(id: &str, amount: &i32, transaction_type: &str, description: &str) -> String {
  format!(
    "INSERT INTO transactions (account_id, amount, transaction_type, description) VALUES ('{}', {}, '{}', '{}')",
    id, amount, transaction_type, description
  )
}

pub fn handle_transaction_request(stream: &mut TcpStream, pg_pool: Pool<PostgresConnectionManager<NoTls>>, client_id: &str, request_body: &str) {
  let pg_pool = pg_pool.clone();
  let client_id: String = client_id.to_string();

  let transaction: Transaction = match serde_json::from_str(request_body) {
    Ok(transaction) => transaction,
    Err(err) => {
      eprintln!("Error deserializing JSON: {}", err);
      let response = Response {
          status: "400 Bad Request".to_string(),
          body: format!("Error deserializing JSON: {}", err),
      };
      handle_response(stream, &response.status, &response.body);
      return;
    }
  };

  let response = Arc::new(Mutex::new(Response {
    status: String::new(),
    body: String::new(),
  }));
  let response_clone = Arc::clone(&response);

  thread::spawn(move || {
    let mut pg_client = pg_pool.get().unwrap();

    let update_result = pg_client.execute(&update_balance_query(&client_id, &transaction.valor,  &transaction.tipo), &[]);

    match update_result {
      Ok(_) => {
        let register_result = pg_client.execute(&register_transaction_query(&client_id, &transaction.valor, &transaction.tipo, &transaction.descricao), &[]);
        match register_result {
          Ok(_) => {
              let mut response = response_clone.lock().unwrap();
              response.status = "200 OK".to_string();
              response.body = "Transaction processed successfully".to_string();
          }
          Err(err) => {
              eprintln!("Error executing registration SQL query: {}", err);
              let mut response = response_clone.lock().unwrap();
              response.status = "404 Not Found".to_string();
              response.body = format!("Error executing registration SQL query: {}", err);
          }
        }
      }
      Err(err) => {
        eprintln!("Error executing update SQL query: {}", err);
        let mut response = response_clone.lock().unwrap();
        response.status = "404 Not Found".to_string();
        response.body = format!("Error executing update SQL query: {}", err);
      }
    }
  }).join().expect("The thread being joined has panicked");

  let response = response.lock().unwrap();
  handle_response(stream, &response.status, &response.body);
}

fn handle_response(tcp_stream: &mut TcpStream, status: &str, content: &str) {
  let response_http = format!("HTTP/1.1 {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}", status, content.len(), content);
  tcp_stream.write_all(response_http.as_bytes()).expect("Failed to write response");
  tcp_stream.flush().expect("Failed to flush");
}