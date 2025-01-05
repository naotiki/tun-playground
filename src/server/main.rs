use std::io;
use tun_playground::protocol::{Protocol, USING_PROTOCOL};
use tun_playground::server::quic::QuicServer;
use tun_playground::server::server::Server;
use tun_playground::server::tcp::TcpServer;
use tun_playground::server::udp::UdpServer;



pub async fn create_server(protocol: Protocol, address: &str) -> io::Result<Box<dyn Server>> {
    match protocol {
        Protocol::Tcp => Ok(Box::new(TcpServer::new(address))),
        Protocol::Udp => Ok(Box::new(UdpServer::new(address))),
        Protocol::Quic => Ok(Box::new(QuicServer::new(address))),
    }
}


#[tokio::main]
async fn main() -> io::Result<()> {
    let protocol = USING_PROTOCOL; // またはProtocol::Udp, Protocol::Quic
    let server = create_server(protocol, "127.0.0.1:8080").await?;
    server.start().await
}
