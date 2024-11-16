use std::{
    error::Error,
    fmt::{Display, Formatter},
};

use async_trait::async_trait;

use crate::{resp::ArrayEntry, storage::Storage};

#[derive(Debug, Clone)]
pub struct CommandError;

impl Display for CommandError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "CommandError")
    }
}

impl Error for CommandError {}

#[async_trait]
pub trait Command: Send + Sync {
    async fn execute(&self, storage: &dyn Storage) -> Result<String, CommandError>;
}

pub struct CommandParser;

impl CommandParser {
    pub fn new(args: Vec<ArrayEntry>) -> Result<Box<dyn Command>, CommandError> {
        let ArrayEntry::Text(cmd) = args.first().unwrap();
        let cmd_kind: Box<dyn Command> = match cmd.as_str() {
            "PING" => Box::new(PingCommand {}),
            "ECHO" => Box::new(EchoCommand {
                args: args[1..]
                    .iter()
                    .map(|ArrayEntry::Text(it)| it.clone())
                    .collect(),
            }),
            "GET" => Box::new(GetCommand {
                key: args
                    .get(1)
                    .map(|ArrayEntry::Text(cmd)| cmd.to_string())
                    .unwrap(),
            }),
            "SET" => Box::new(SetCommand {
                key: args
                    .get(1)
                    .map(|ArrayEntry::Text(cmd)| cmd.to_string())
                    .unwrap(),
                value: args
                    .get(2)
                    .map(|ArrayEntry::Text(cmd)| cmd.to_string())
                    .unwrap(),
            }),
            _ => return Err(CommandError),
        };
        Ok(cmd_kind)
    }
}

pub struct GetCommand {
    key: String,
}

#[async_trait]
impl Command for GetCommand {
    async fn execute(&self, storage: &dyn Storage) -> Result<String, CommandError> {
        println!("GET was called");
        match storage.get(&self.key).await {
            Some(value) => {
                let msg = format!("+{}\r\n", value);
                Ok(msg)
            }
            None => Ok("$-1\r\n".to_string()),
        }
    }
}

pub struct PingCommand;

#[async_trait]
impl Command for PingCommand {
    async fn execute(&self, _: &dyn Storage) -> Result<String, CommandError> {
        println!("PING was called");
        Ok(String::from("+PONG\r\n"))
    }
}

#[derive(Debug)]
pub struct EchoCommand {
    args: Vec<String>,
}

#[async_trait]
impl Command for EchoCommand {
    async fn execute(&self, _: &dyn Storage) -> Result<String, CommandError> {
        println!("ECHO was called: {:?}", self);
        let msg = format!("+{}\r\n", self.args.join("\r\n"));
        Ok(msg)
    }
}

#[derive(Debug)]
pub struct SetCommand {
    key: String,
    value: String,
}

#[async_trait]
impl Command for SetCommand {
    async fn execute(&self, storage: &dyn Storage) -> Result<String, CommandError> {
        storage.set(self.key.clone(), self.value.clone()).await;
        Ok("+OK\r\n".to_string())
    }
}
