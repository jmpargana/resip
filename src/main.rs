use std::sync::Arc;

use redis_starter_rust::server::Server;
use redis_starter_rust::storage::InMemoryStorage;

#[tokio::main]
async fn main() {
    let storage = InMemoryStorage::new();
    let server = Server::new(Arc::new(storage));
    server.run("127.0.0.1:6379").await.expect("Server failed");
}
