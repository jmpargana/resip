use std::{
    error::Error,
    fmt::{Display, Formatter},
    time::{Duration, Instant},
};

use async_trait::async_trait;

use crate::{
    resp::ArrayEntry,
    storage::{Storage, Value},
};

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
        // Extract the first argument (command name)
        let cmd = match args.first() {
            Some(ArrayEntry::Text(cmd)) => cmd.as_str(),
            _ => return Err(CommandError), // Return an error if the command name is missing or invalid
        };

        // Match the command name to create the corresponding command
        let cmd_kind: Box<dyn Command> = match cmd {
            "PING" => Box::new(PingCommand {}),

            "ECHO" => {
                let echo_args = args
                    .iter()
                    .skip(1)
                    .filter_map(|entry| match entry {
                        ArrayEntry::Text(text) => Some(text.clone()),
                        _ => None, // Skip non-text arguments
                    })
                    .collect();

                Box::new(EchoCommand { args: echo_args })
            }

            "GET" => {
                let key = args
                    .get(1)
                    .and_then(|entry| match entry {
                        ArrayEntry::Text(text) => Some(text.to_string()),
                        _ => None,
                    })
                    .ok_or(CommandError)?; // Return an error if key is missing or invalid

                Box::new(GetCommand { key })
            }

            "SET" => {
                let key = args
                    .get(1)
                    .and_then(|entry| match entry {
                        ArrayEntry::Text(text) => Some(text.to_string()),
                        _ => None,
                    })
                    .ok_or(CommandError)?;

                let value = args
                    .get(2)
                    .and_then(|entry| match entry {
                        ArrayEntry::Text(text) => Some(text.to_string()),
                        _ => None,
                    })
                    .ok_or(CommandError)?;

                let expiry = if args.len() == 5 {
                    args.get(4).and_then(|entry| match entry {
                        ArrayEntry::Text(number) => {
                            let number = number.parse::<u64>().expect("could not parse number");
                            Some(Instant::now() + Duration::from_millis(number))
                        }
                        _ => None,
                    })
                } else {
                    None
                };

                Box::new(SetCommand { key, value, expiry })
            }

            _ => return Err(CommandError), // Unknown command
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
                let msg = format!("+{}\r\n", value.value);
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
    expiry: Option<Instant>,
}

#[async_trait]
impl Command for SetCommand {
    async fn execute(&self, storage: &dyn Storage) -> Result<String, CommandError> {
        storage
            .set(
                self.key.clone(),
                Value {
                    value: self.value.clone(),
                    expiry: self.expiry,
                },
            )
            .await;
        Ok("+OK\r\n".to_string())
    }
}
