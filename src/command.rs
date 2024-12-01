use std::{
    error::Error,
    fmt::{Display, Formatter},
    time::{Duration, Instant},
};

use async_trait::async_trait;

use crate::{
    resp::{Array, Entry},
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

fn parse_arg(args: &[Entry], at: usize) -> Result<String, CommandError> {
    args.get(at)
        .and_then(|entry| match entry {
            Entry::Text(text) => Some(text.to_string()),
            _ => None,
        })
        .ok_or(CommandError)
}

pub struct CommandParser;

impl CommandParser {
    pub fn new(args: &[Entry]) -> Result<Box<dyn Command>, CommandError> {
        // Extract the first argument (command name)
        let cmd = match args.first() {
            Some(Entry::Text(cmd)) => cmd.as_str(),
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
                        Entry::Text(text) => Some(text.clone()),
                        _ => None, // Skip non-text arguments
                    })
                    .collect();

                Box::new(EchoCommand { args: echo_args })
            }

            "GET" => Box::new(GetCommand {
                key: parse_arg(args, 1)?,
            }),

            "SET" => {
                let key = parse_arg(args, 1)?;
                let value = parse_arg(args, 2)?;

                let expiry = if args.len() == 5 {
                    args.get(4).and_then(|entry| match entry {
                        Entry::Text(number) => {
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

            "CONFIG" => {
                let key = parse_arg(args, 2)?;
                Box::new(ConfigGetCommand { key })
            }

            "SAVE" => Box::new(SaveCommand),

            "KEYS" => {
                let key = parse_arg(args, 1)?;
                Box::new(KeysCommand { key })
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
        match storage.get(&self.key).await {
            Some(value) => {
                let msg = Entry::SimpleText(value.value.to_string());
                Ok(msg.to_string())
            }
            None => Ok(Entry::Nil.to_string()),
        }
    }
}

pub struct PingCommand;

#[async_trait]
impl Command for PingCommand {
    async fn execute(&self, _: &dyn Storage) -> Result<String, CommandError> {
        Ok(Entry::SimpleText("PONG".to_string()).to_string())
    }
}

#[derive(Debug)]
pub struct EchoCommand {
    args: Vec<String>,
}

#[async_trait]
impl Command for EchoCommand {
    async fn execute(&self, _: &dyn Storage) -> Result<String, CommandError> {
        let msg = Entry::SimpleText(self.args.join("\r\n"));
        Ok(msg.to_string())
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
        Ok(Entry::SimpleText("OK".to_string()).to_string())
    }
}

pub struct ConfigGetCommand {
    key: String,
}

#[async_trait]
impl Command for ConfigGetCommand {
    async fn execute(&self, storage: &dyn Storage) -> Result<String, CommandError> {
        let config = storage.config().await;
        match self.key.as_str() {
            "dir" => {
                let msg = Array(vec![
                    Entry::Text(self.key.to_string()),
                    Entry::Text(config.dir),
                ]);
                Ok(msg.to_string())
            }
            "dbfilename" => {
                let msg = Array(vec![
                    Entry::Text(self.key.to_string()),
                    Entry::Text(config.path),
                ]);
                Ok(msg.to_string())
            }
            _ => Ok(Entry::Nil.to_string()),
        }
    }
}

pub struct SaveCommand;

#[async_trait]
impl Command for SaveCommand {
    async fn execute(&self, storage: &dyn Storage) -> Result<String, CommandError> {
        storage.save().await.map_err(|_| CommandError)?;
        Ok(Entry::Nil.to_string())
    }
}

pub struct KeysCommand {
    key: String,
}

#[async_trait]
impl Command for KeysCommand {
    async fn execute(&self, storage: &dyn Storage) -> Result<String, CommandError> {
        match storage.keys(&self.key).await {
            None => Ok(Entry::Nil.to_string()),
            Some(v) => Ok(Array(v.iter().map(|k| Entry::Text(k.clone())).collect()).to_string()),
        }
    }
}
