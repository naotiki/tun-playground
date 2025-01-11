use crate::server::server::Server;
use std::io;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use super::server::{AppSession, AsyncSessionHandler, };

pub struct TcpServer {
    address: String,
}

impl TcpServer {
    pub fn new(address: &str) -> Self {
        Self {
            address: address.to_string(),
        }
    }

    async fn handle_client(mut stream: TcpStream) -> io::Result<()> {
        let mut buffer = vec![0; 1024];
        loop {
            let n = stream.read(&mut buffer).await?;
            if n == 0 {
                break;
            }
            
            println!("Received: {:?}", &buffer[..n]);
            stream.write_all(&buffer[..n]).await?; // Echo back the data
        }
        Ok(())
    }
}

#[async_trait::async_trait]
impl Server for TcpServer {
    async fn start(&self,session_handler: AsyncSessionHandler) -> io::Result<()> 
   
    {
        let listener = TcpListener::bind(&self.address).await?;
        println!("TCP server listening on {}", self.address);
        
        loop {
            let (stream, addr) = listener.accept().await?;
            println!("New connection from {}", addr);
            // Handle each client in a separate task
            tokio::spawn(async move {
                let (read, write) = stream.into_split();
                let session = AppSession::new(Box::new(read), Box::new(write));
                if let Err(e) = session_handler(session).await {
                    eprintln!("Failed to handle client: {}", e);
                }
            });
        }
    }
}
