use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, BufReader},
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpStream,
    },
};

#[derive(Debug, Clone)]
pub struct ConnectionError;

pub struct Connection {
    reader: BufReader<OwnedReadHalf>,
    writer: OwnedWriteHalf,
}

impl Connection {
    pub fn new(stream: TcpStream) -> Connection {
        let (reader, writer) = stream.into_split();
        let reader = BufReader::new(reader);
        Connection { reader, writer }
    }

    pub async fn read_command(&mut self) -> Result<Option<String>, ConnectionError> {
        let mut buffer = vec![0u8; 112]; // TODO: change size
        let n = self
            .reader
            .read(&mut buffer)
            .await
            .map_err(|_| ConnectionError)?;

        if n == 0 {
            return Ok(None);
        }
        let msg = String::from_utf8_lossy(&buffer);

        if msg.is_empty() {
            return Ok(None);
        }
        Ok(Some(msg.to_string()))
    }

    pub async fn send_response(&mut self, content: &str) -> Result<(), ConnectionError> {
        println!("response being sent: {:?}", content);
        self.writer
            .write_all(content.as_bytes())
            .await
            .map_err(|_| ConnectionError)?;
        self.writer.flush().await.map_err(|_| ConnectionError)?;
        Ok(())
    }
}
