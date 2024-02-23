use std::net::TcpStream;
use std::io::Write;
use std::thread;
use std::sync::{Arc, Mutex};
use chrono::{self, Date, DateTime, Local};
use r2d2_postgres::{postgres::NoTls, PostgresConnectionManager};
use r2d2::Pool;
use serde::{Deserialize, Serialize};
use serde_json;

struct Response {
  status: String,
  body: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct Transaction {
  valor: i32,
  tipo: String,
  descricao: String,
  realizada_em: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct Balance {
  total: i32,
  data_extrato: String,
  limite: i32,
}

fn get_balance_query(id: &str) -> String {format!("SELECT a.limit_amount, b.amount FROM accounts AS a JOIN balances AS b ON a.id = b.account_id WHERE a.id = {}", id)}
fn get_last_10_transactions_query(id: &str) -> String {
  format!("SELECT amount, transaction_type, description, date FROM transactions WHERE account_id = '{}' ORDER BY id DESC LIMIT 10", id)
}

pub fn handle_extract_request(stream: &mut TcpStream, pg_pool: Pool<PostgresConnectionManager<NoTls>>, client_id: &str) {
    let pool = pg_pool.clone();
    let id_clone: String = client_id.to_string();

    let response = Arc::new(Mutex::new(Response {
      status: String::new(),
      body: String::new(),
    }));
    let response_clone = Arc::clone(&response);

    thread::spawn(move || {
      let mut client = pool.get().unwrap();
      
      let balance_result = client.query_one(&get_balance_query(&id_clone), &[]);
      let transactions_result = client.query(&get_last_10_transactions_query(&id_clone), &[]);

      match (balance_result, transactions_result) {
        (Ok(balance_row), Ok(transactions_rows)) => {
            let limit: i32 = balance_row.get(0);
            let time_now = chrono::offset::Local::now().to_string();
            let balance: i32 = balance_row.get(1);
            let account_balance = Balance {
              total: limit,
              data_extrato: time_now,
              limite: balance
            };

            let balance_json = serde_json::to_string(&account_balance).unwrap();

            let transactions: Vec<Transaction> = transactions_rows
                .iter()
                .map(|row| Transaction {
                    valor: row.get(1),
                    tipo: row.get(2),
                    descricao: row.get(3),
                    realizada_em: row.get(4),
                })
                .collect();
            
            let transactions_json = serde_json::to_string(&transactions).unwrap();

            let combined_data = format!(
                r#"{{ "saldo": {}, "ultimas_transacoes": {} }}"#,
                balance_json, transactions_json
            );

            let mut response = response_clone.lock().unwrap();
            response.body = combined_data.to_string();
            response.status = format!("200 OK");
        }
        (Err(balance_err), _) => {
            eprintln!("Failed to retrieve balance for client {}: {}", id_clone, balance_err);
            let mut response = response_clone.lock().unwrap();
            response.body = format!(r#"{{ "error": "Failed to retrieve balance for client {}" }}"#, id_clone);
            response.status = format!("404 Not Found");
        }
        (_, Err(transactions_err)) => {
            eprintln!("Failed to retrieve transactions for client {}: {}", id_clone, transactions_err);
            let mut response = response_clone.lock().unwrap();
            response.body = format!(r#"{{ "error": "Failed to retrieve transactions for client {}" }}"#, id_clone);
            response.status = format!("404 Not Found");
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