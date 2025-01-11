use std::io;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tun_playground::protocol::{Protocol, USING_PROTOCOL};
use tun_playground::server::quic::QuicServer;
use tun_playground::server::server::Server;
use tun_playground::server::tcp::TcpServer;


pub async fn create_server(protocol: Protocol, address: &str) -> io::Result<Box<dyn Server>> {
    match protocol {
        Protocol::Tcp => Ok(Box::new(TcpServer::new(address))),
        Protocol::Quic => Ok(Box::new(QuicServer::new(address))),
    }
}


#[tokio::main]
async fn main() -> io::Result<()> {
    let protocol = USING_PROTOCOL; // またはProtocol::Udp, Protocol::Quic
    let server = create_server(protocol, "127.0.0.1:8080").await?;
    server.start(|session| Box::pin(async move {
        println!("Session ID: {}", session.session_id);
        let mut read = session.read;
        let mut write = session.write;
        let mut buffer = vec![0; 1024];
        loop {
            let n = read.read(&mut buffer).await?;
            if n == 0 {
                break;
            }
            
            println!("Received: {:?}", &buffer[..n]);
            write.write_all(&buffer[..n]).await?; // Echo back the data
        }
        Ok(())
    })).await
}
