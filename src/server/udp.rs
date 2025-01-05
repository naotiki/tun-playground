use std::io;
use tokio::net::UdpSocket;
use crate::server::server::Server;

pub struct UdpServer {
    address: String,
}

impl UdpServer {
    pub fn new(address: &str) -> Self {
        Self {
            address: address.to_string(),
        }
    }
}

#[async_trait::async_trait]
impl Server for UdpServer {
    async fn start(&self) -> io::Result<()> {
        let socket = UdpSocket::bind(&self.address).await?;
        println!("UDP server listening on {}", self.address);

        let mut buffer = vec![0; 1024];
        loop {
            let (n, addr) = socket.recv_from(&mut buffer).await?;
            println!("Received from {}: {:?}", addr, &buffer[..n]);
            socket.send_to(&buffer[..n], addr).await?; // Echo back the data
        }
    }
}
