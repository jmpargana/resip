use std::env;
use std::sync::Arc;

use redis_starter_rust::server::Server;
use redis_starter_rust::storage::{InMemoryStorage, Storage, Value};

#[tokio::main]
async fn main() {
    let storage = InMemoryStorage::new();

    let args: Vec<String> = env::args().skip(1).collect();
    let mut args_iter = args.iter();
    while let Some(s) = args_iter.next() {
        if let Some(value) = args_iter.next() {
            let key_trimmed = s.trim_start_matches("--");
            storage
                .set(
                    key_trimmed.to_string(),
                    Value {
                        value: value.to_string(),
                        expiry: None,
                    },
                )
                .await;
        }
    }

    let server = Server::new(Arc::new(storage));
    server.run("127.0.0.1:6379").await.expect("Server failed");
}
