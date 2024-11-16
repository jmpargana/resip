use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::thread;
use tokio::time::{sleep, Duration};

use redis_starter_rust::start_server;

const ADDR: &str = "127.0.0.1:6379";

#[tokio::test]
async fn test_connection_is_accepted() {
    thread::spawn(|| {
        start_server(ADDR); // Start server in separate thread
    });

    sleep(Duration::from_millis(100)).await;

    let client = TcpStream::connect("127.0.0.1:6379");
    assert!(client.is_ok(), "Expected connection to be accepted");
}

#[tokio::test]
async fn test_if_ping_gets_pong() {
    thread::spawn(|| {
        start_server(ADDR); // Start server in separate thread
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
        start_server(ADDR); // Start server in separate thread
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
        start_server(ADDR); // Start server in separate thread
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
