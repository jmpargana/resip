use async_trait::async_trait;
use std::collections::HashMap;
use tokio::sync::RwLock;

#[async_trait]
pub trait Storage: Send + Sync {
    async fn set(&self, key: String, value: String);
    async fn get(&self, key: &str) -> Option<String>;
}

pub struct InMemoryStorage {
    map: RwLock<HashMap<String, String>>,
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
    async fn set(&self, key: String, value: String) {
        self.map.write().await.insert(key, value);
    }
    async fn get(&self, key: &str) -> Option<String> {
        self.map.read().await.get(key).cloned()
    }
}
