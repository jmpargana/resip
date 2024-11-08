#![allow(unused_imports)]
use std::io::BufRead;

use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
    task,
};

async fn handle_client(mut _stream: TcpStream) {
    let (reader, mut writer) = _stream.into_split();
    let mut reader = BufReader::new(reader);
    let mut msg: Vec<String> = vec![];
    let mut found_echo = false;
    let mut echo_index = 0;

    loop {
        let mut buffer = String::new();

        match reader.read_line(&mut buffer).await {
            Ok(0) => {
                println!("client disconnected");
                break;
            }
            Ok(_) => {
                msg.push(buffer);
            }
            Err(e) => {
                println!("failed reading, {}", e);
                break;
            }
        }

        while let Some(m) = msg.pop() {
            if m == "PING\r\n" {
                writer.write_all(b"+PONG\r\n").await;
                writer.flush().await;
            }
            if m == "ECHO\r\n" {
                found_echo = true;
            }
            if found_echo {
                echo_index += 1;
            }
            if found_echo && echo_index == 3 {
                let resp = format!("+{}", m);
                writer.write_all(resp.as_bytes()).await;
                writer.flush().await;
            }
        }
    }
}

#[tokio::main]
async fn main() {
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:6379")
        .await
        .expect("failed to bind");

    loop {
        let (stream, _) = listener.accept().await.expect("failed to accept listener");
        task::spawn(async move {
            handle_client(stream).await;
        });
    }
}

#[cfg(test)]
mod tests {
    use std::io::{BufRead, BufReader, Read, Write};
    use std::net::TcpStream;
    use std::thread;
    use tokio::time::{sleep, Duration};

    use super::*;

    #[tokio::test]
    async fn test_connection_is_accepted() {
        thread::spawn(|| {
            main(); // Start server in separate thread
        });

        sleep(Duration::from_millis(100)).await;

        let client = TcpStream::connect("127.0.0.1:6379");
        assert!(client.is_ok(), "Expected connection to be accepted");
    }

    #[tokio::test]
    async fn test_if_ping_gets_pong() {
        thread::spawn(|| {
            main(); // Start server in separate thread
        });

        sleep(Duration::from_millis(100)).await;

        let mut client = TcpStream::connect("127.0.0.1:6379").expect("failed to connect");
        client.write_all(b"PING\r\n").expect("failed to write");

        let mut reader = BufReader::new(client);
        let mut response = String::new();
        reader.read_line(&mut response).expect("Failed to read");

        assert_eq!(response, "+PONG\r\n");
    }

    #[tokio::test]
    async fn test_if_multiple_pings_get_pongs() {
        thread::spawn(|| {
            main(); // Start server in separate thread
        });

        sleep(Duration::from_millis(100)).await;

        let mut client = TcpStream::connect("127.0.0.1:6379").expect("failed to connect");

        for _ in 0..3 {
            client.write_all(b"PING\r\n").expect("Failed to write");

            let mut reader = BufReader::new(client.try_clone().expect("Failed to clone client"));
            let mut response = String::new();
            reader.read_line(&mut response).expect("Failed to read");

            assert_eq!(response.trim(), "+PONG", "Expected response to be +PONG");
        }
    }

    #[tokio::test]
    async fn test_reply_to_echo() {
        thread::spawn(|| {
            main(); // Start server in separate thread
        });

        sleep(Duration::from_millis(100)).await;

        let mut client = TcpStream::connect("127.0.0.1:6379").expect("failed to connect");
        client
            .write_all(b"*2\r\n$4\r\nECHO\r\n$3\r\nhey\r\n")
            .expect("failed to write");

        let mut reader = BufReader::new(client);
        let mut response = String::new();
        reader.read_line(&mut response).expect("Failed to read");

        assert_eq!(response, "+hey\r\n");
    }
}
