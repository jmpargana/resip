use std::error::Error;
use std::sync::Arc;
use std::time::Duration;

use clap::Parser;
use redis_starter_rust::server::Server;
use redis_starter_rust::storage::{InMemoryStorage, RdbStorage, Storage};
use tokio::sync::Mutex;
use tokio::task;
use tokio::time::sleep;

#[derive(Parser, Debug)]
// #[command(version, about, long_about = None)]
struct Args {
    #[arg(long)]
    dir: Option<String>,
    #[arg(long)]
    dbfilename: Option<String>,
    #[arg(long, default_value_t = 6379)]
    port: u32,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let storage: Arc<Mutex<dyn Storage>> = if args.dir == None || args.dbfilename == None {
        let storage = InMemoryStorage::new();
        Arc::new(Mutex::new(storage))
    } else {
        let mut storage = RdbStorage::new(&args.dir.unwrap(), &args.dbfilename.unwrap());
        storage.load().await.unwrap();
        let storage = Arc::new(Mutex::new(storage));
        let storage_clone = Arc::clone(&storage);

        task::spawn(async move {
            loop {
                let _ = sleep(Duration::from_secs(60));
                let storage_guard = storage.lock().await;
                let _ = storage_guard.save().await;
            }
        });
        storage_clone
    };

    let server = Server::new(storage);
    server
        .run(&format!("127.0.0.1:{}", args.port))
        .await
        .expect("Server failed");
    Ok(())
}
