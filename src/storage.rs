use async_trait::async_trait;
use std::{collections::HashMap, time::Instant};
use tokio::sync::RwLock;

#[derive(Clone, Debug)]
pub struct Value {
    pub value: String,
    pub expiry: Option<Instant>,
}

#[async_trait]
pub trait Storage: Send + Sync {
    async fn set(&self, key: String, value: Value);
    async fn get(&self, key: &str) -> Option<Value>;
}

pub struct InMemoryStorage {
    map: RwLock<HashMap<String, Value>>,
}

impl InMemoryStorage {
    pub fn new() -> InMemoryStorage {
        InMemoryStorage {
            map: RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl Storage for InMemoryStorage {
    async fn set(&self, key: String, value: Value) {
        self.map.write().await.insert(key, value);
    }

    async fn get(&self, key: &str) -> Option<Value> {
        let value = self.map.read().await.get(key)?.clone();

        println!("value found {:?}", value);

        if let Some(expiry) = value.expiry {
            if Instant::now() > expiry {
                return None;
            }
        }

        Some(value)
    }
}
