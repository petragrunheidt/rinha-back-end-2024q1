use std::net::TcpStream;
use std::io::Write;
use std::thread;
use std::sync::{Arc, Mutex};
use chrono;
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
  realizada_em: String,
}

fn get_balance_query(id: &str) -> String {format!("SELECT a.limit_amount, b.amount FROM accounts AS a JOIN balances AS b ON a.id = b.account_id WHERE a.id = {}", id)}
fn update_balance_query(amount: &str, id: &str) -> String {format!("UPDATE accounts SET limit_amount = limit_amount - {} WHERE id = {}", amount, id)}
fn register_transaction_query(id: &str, amount: &str, transaction_type: &str, description: &str) -> String {
  format!(
    "INSERT INTO transactions (account_id, amount, transaction_type, description) VALUES ({}, {}, {}, {})",
    id, amount, transaction_type, description
  )
}
fn get_last_10_transactions_query(id: &str) -> String {
  format!("SELECT account_id, amount, transaction_type, description FROM transactions WHERE account_id = '{}' ORDER BY transaction_id DESC LIMIT 10", id)
}


pub fn handle_transaction_request(stream: &mut TcpStream, pg_pool: Pool<PostgresConnectionManager<NoTls>>, client_id: &str, request_body: &str) {
  // let pool = pg_pool.clone();
  // let client_id_clone: String = client_id.to_string();

  // println!("{}", request_body);

  // let transaction: Transaction = match serde_json::from_str(request_body) {
  //   Ok(transaction) => transaction,
  //   Err(err) => {
  //     eprintln!("Error deserializing JSON: {}", err);
  //     let response = Response {
  //         status: "400 Bad Request".to_string(),
  //         body: format!("Error deserializing JSON: {}", err),
  //     };
  //     handle_response(stream, &response.status, &response.body);
  //     return;
  //   }
  // };

  // let response = Arc::new(Mutex::new(Response {
  //   status: String::new(),
  //   body: String::new(),
  // }));
  // let response_clone = Arc::clone(&response);

  // thread::spawn(move || {
  //   let mut client = pool.get().unwrap();

  //   // let balance_result = client.query_one(&get_balance_query(cliend_id_clone), &[]);
  //   // let limit_amount: i32 = row.get(0);
  //   // let amount: i32 = row.get(1);

  //   let result = match transaction.tipo {
  //     'c' => {
  //         client.execute(&update_balance_query(&transaction.amount, &client_id_clone), &[]);
  //         client.execute(
  //           "INSERT INTO transactions (account_id, amount, transaction_type, description) VALUES ($1, $2, $3, $4)",
  //           &[&client_id_clone, &transaction.valor, &transaction.tipo &transaction.descricao],
  //       );
  //     }
  //     'd' => {
  //         client.execute(
  //             "UPDATE accounts SET amount = limit_amount - $1 WHERE id = $2",
  //             &[&transaction.amount, &client_id_clone],
  //         );
  //         client.execute(
  //           "INSERT INTO transactions (account_id, amount, transaction_type, description) VALUES ($1, $2, $3, $4)",
  //           &[&client_id_clone, &transaction.valor, &transaction.tipo, &transaction.descricao],
  //       );
  //     }
  //     _ => {
  //         eprintln!("Invalid transaction type");
  //         Err(postgres::Error::Io(std::io::Error::new(
  //             std::io::ErrorKind::Other,
  //             "Invalid transaction type",
  //         )))
  //     }
  // };

  // match result {
  //     Ok(_) => {
  //         let mut response = response_clone.lock().unwrap();
  //         response.status = "200 OK".to_string();
  //         response.body = "Transaction processed successfully".to_string();
  //     }
  //     Err(err) => {
  //         eprintln!("Error executing SQL query: {}", err);
  //         let mut response = response_clone.lock().unwrap();
  //         response.status = "404 Not Found".to_string();
  //         response.body = format!("Error executing SQL query: {}", err);
  //     }
  //   }
  // }).join().expect("The thread being joined has panicked");

  // let response = response.lock().unwrap();
  // handle_response(stream, &response.status, &response.body);
}

fn handle_response(tcp_stream: &mut TcpStream, status: &str, content: &str) {
  let response_http = format!("HTTP/1.1 {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}", status, content.len(), content);
  tcp_stream.write_all(response_http.as_bytes()).expect("Failed to write response");
  tcp_stream.flush().expect("Failed to flush");
}