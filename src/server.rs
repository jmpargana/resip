use crate::command::CommandParser;
use crate::connection::Connection;
use crate::resp::*;
use crate::storage::Storage;
use pest::Parser;
use std::sync::Arc;
use tokio::{net::TcpListener, task};

#[derive(Debug, Clone)]
pub struct ServerError;

pub struct Server {
    storage: Arc<dyn Storage>,
}

impl Server {
    pub fn new(storage: Arc<dyn Storage>) -> Self {
        Server { storage }
    }

    pub async fn run(&self, addr: &str) -> Result<(), ServerError> {
        println!("Logs from your program will appear here!");
        let listener = TcpListener::bind(addr).await.expect("failed to bind");

        loop {
            let (stream, _) = listener.accept().await.expect("failed to accept listener");

            let storage = Arc::clone(&self.storage);
            task::spawn(async move {
                let mut connection = Connection::new(stream);

                loop {
                    if let Some(str) = connection.read_command().await.unwrap() {
                        let result = RESPParser::parse(Rule::array, &str)
                            .expect("failed step 1 of parsing")
                            .next()
                            .expect("failed step 2 of parsing");

                        let entries = extract_array_entries(result);

                        let cmd = match CommandParser::new(entries) {
                            Ok(command) => command,
                            Err(_) => {
                                connection
                                    .send_response("-ERR unknown command\r\n")
                                    .await
                                    .expect("failed to send error");
                                continue;
                            }
                        };

                        let msg = cmd
                            .execute(storage.as_ref())
                            .await
                            .expect("failed executing command");
                        connection
                            .send_response(&msg)
                            .await
                            .expect("failed to send response");
                    } else {
                        println!("no message, continuing...");
                        break;
                    }
                }
            });
        }
    }
}
