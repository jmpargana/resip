use crate::rdb::{parse_rdb_file, write_rdb_file};
use async_trait::async_trait;
use regex::Regex;
use std::{collections::HashMap, io, time::Instant};
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
    async fn save(&self) -> Result<(), io::Error>;
    async fn load(&mut self) -> Result<(), io::Error>;
    async fn keys(&self, key: &str) -> Option<Vec<String>>;
    async fn config(&self) -> RdbConfig;
}

#[derive(Clone, Debug)]
pub struct RdbConfig {
    pub dir: String,
    pub path: String,
}

impl RdbConfig {
    fn config_file(&self) -> String {
        format!("{}/{}", self.dir, self.path)
    }
}

#[derive(Debug)]
pub struct InMemoryStorage {
    map: RwLock<HashMap<String, Value>>,
}

impl InMemoryStorage {
    pub fn new() -> Self {
        Self {
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
        if let Some(expiry) = value.expiry {
            if Instant::now() > expiry {
                return None;
            }
        }
        Some(value)
    }

    async fn save(&self) -> Result<(), io::Error> {
        Ok(())
    }

    async fn load(&mut self) -> Result<(), io::Error> {
        Ok(())
    }

    async fn keys(&self, k: &str) -> Option<Vec<String>> {
        let map = self.map.read().await;
        let keys: Vec<&str> = map.keys().map(String::as_str).collect();
        Some(
            needle_in_haystack(k, &keys)
                .into_iter()
                .map(String::from)
                .collect(),
        )
    }

    async fn config(&self) -> RdbConfig {
        RdbConfig {
            dir: "".to_string(),
            path: "".to_string(),
        }
    }
}

#[derive(Debug)]
pub struct RdbStorage {
    config: RdbConfig,
    map: RwLock<HashMap<String, Value>>,
}

impl RdbStorage {
    pub fn new(dir: &str, path: &str) -> Self {
        let dir = dir.to_string();
        let path = path.to_string();
        Self {
            config: RdbConfig { dir, path },
            map: RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl Storage for RdbStorage {
    async fn set(&self, key: String, value: Value) {
        self.map.write().await.insert(key, value);
    }

    async fn get(&self, key: &str) -> Option<Value> {
        let value = self.map.read().await.get(key)?.clone();
        if let Some(expiry) = value.expiry {
            if Instant::now() > expiry {
                return None;
            }
        }
        Some(value)
    }

    async fn save(&self) -> Result<(), io::Error> {
        let m = self.map.read().await.clone();
        write_rdb_file(&self.config.config_file(), m)
    }

    async fn load(&mut self) -> Result<(), io::Error> {
        println!("loading file... {:?}", self.config);
        let map = parse_rdb_file(&self.config.config_file())
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "failed parsing file"))
            .unwrap();
        self.map = RwLock::new(map);
        Ok(())
    }

    async fn keys(&self, k: &str) -> Option<Vec<String>> {
        let map = self.map.read().await;
        let keys: Vec<&str> = map.keys().map(String::as_str).collect();
        Some(
            needle_in_haystack(k, &keys)
                .into_iter()
                .map(String::from)
                .collect(),
        )
    }

    async fn config(&self) -> RdbConfig {
        self.config.clone()
    }
}

fn needle_in_haystack<'a>(key: &str, haystack: &[&'a str]) -> Vec<&'a str> {
    let mut needle = String::new();
    for ch in key.chars() {
        if ch == '*' {
            needle.push('.');
        }
        needle.push(ch);
    }
    let re = Regex::new(&needle).unwrap();
    haystack
        .iter()
        .map(|&it| it)
        .filter(|&it| re.is_match(it))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_match_asterisk() {
        let needle = "*";
        let haystack = vec!["foo", "bar"];

        let actual = needle_in_haystack(needle, &haystack);
        assert_eq!(actual, haystack);
    }

    #[ignore]
    #[test]
    fn should_match_partial_asterisk() {
        let needle = "f*";
        let haystack = vec!["foo", "bar"];

        let actual = needle_in_haystack(needle, &haystack);
        assert_eq!(actual, vec!["foo"]);
    }
}
