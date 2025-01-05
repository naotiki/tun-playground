use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::sync::{Arc,};

use std::io;
use async_trait::async_trait;
use tokio::sync::Mutex;
use crate::client::transport::Transport;

pub struct TcpTransport {
    stream: Arc<Mutex<TcpStream>>, // Arc<Mutex<TcpStream>>を使用
}

impl TcpTransport {
    pub async fn new(address: &str) -> io::Result<Self> {
        let stream = TcpStream::connect(address).await?;
        Ok(Self {
            stream: Arc::new(Mutex::new(stream)),
        })
    }
}
#[async_trait]
impl Transport for TcpTransport {
    async fn send(&self, data: &[u8]) -> io::Result<()> {
        let mut stream = self.stream.lock().await; // Mutexをロックして可変参照を取得
        stream.write_all(data).await
    }

    async fn receive(&self) -> io::Result<Vec<u8>> {
        let mut stream = self.stream.lock().await; // Mutexをロックして可変参照を取得
        let mut buffer = vec![0; 1024];
        let n = stream.read(&mut buffer).await?;
        buffer.truncate(n);
        Ok(buffer)
    }
}
