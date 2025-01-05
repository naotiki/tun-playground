use std::io;
use tokio::net::UdpSocket;
use crate::client::transport::Transport;

pub struct UdpTransport {
    socket: UdpSocket,
    remote_addr: String,
}

impl UdpTransport {
    pub async fn new(bind_addr: &str, remote_addr: &str) -> io::Result<Self> {
        let socket = UdpSocket::bind(bind_addr).await?;
        socket.connect(remote_addr).await?;
        Ok(Self {
            socket,
            remote_addr: remote_addr.to_string(),
        })
    }
}
#[async_trait::async_trait]
impl Transport for UdpTransport {
    async fn send(&self, data: &[u8]) -> io::Result<()> {
        self.socket.send(data).await.map(|_| ())
    }

    async fn receive(&self) -> io::Result<Vec<u8>> {
        let mut buffer = vec![0; 1024];
        let n = self.socket.recv(&mut buffer).await?;
        buffer.truncate(n);
        Ok(buffer)
    }
}
