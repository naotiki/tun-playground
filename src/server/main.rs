use bollard::network::CreateNetworkOptions;
use bollard::Docker;
use futures::{SinkExt, StreamExt};
use tun_playground::tun::TunInterface;
use std::collections::HashMap;
use std::io;
use std::net::Ipv4Addr;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tun_playground::protocol::{Protocol, USING_PROTOCOL};
use tun_playground::server::quic::QuicServer;
use tun_playground::server::server::Server;
use tun_playground::server::tcp::TcpServer;

#[cfg(not(target_os = "windows"))]
use tappers::tokio::AsyncTap;

pub async fn create_server(protocol: Protocol, address: &str) -> io::Result<Box<dyn Server>> {
    match protocol {
        Protocol::Tcp => Ok(Box::new(TcpServer::new(address))),
        Protocol::Quic => Ok(Box::new(QuicServer::new(address))),
    }
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let protocol = USING_PROTOCOL; // またはProtocol::Udp, Protocol::Quic
    let server = create_server(protocol, "0.0.0.0:8080").await?;

/*
    let docker = Docker::connect_with_defaults().unwrap();
    // create new docker network
     let network = docker
        .create_network(CreateNetworkOptions {
            name: "tunnel-network".to_string(),
            driver: "ipvlan".to_string(),
            options: HashMap::from([
                ("subnet".to_string(), "10.0.0.0/24".to_string()),
                ("gateway".to_string(), "10.0.0.1".to_string()),
                ("ipvlan_mode".to_string(), "l2".to_string()),
                ("parent".to_string(), "eth0".to_string()),
            ]),
            internal: true,
            ..Default::default()
        })
        .await
        .unwrap(); */
    // without windows
    #[cfg(not(target_os = "windows"))]
    {
        server
        .start(|session| {
            Box::pin(async move {
                let mut tap = AsyncTap::new()?;
                tap.add_addr(Ipv4Addr::new(10, 0, 0, 1))?;
                tap.set_up()?;
                let tap_writer = Arc::new(tap);
                let tap_reader = tap_writer.clone();

                println!("Session ID: {}", session.session_id);
                let mut quic_reader = session.read;
                let mut quic_writer = session.write;
                tokio::spawn(async move {
                    let mut buffer = vec![0; 1024];
                    loop {
                        let response = quic_reader.read(&mut buffer).await.unwrap();
                        tap_writer.send(&buffer[..response]).await.unwrap();
                    }
                });
                let mut buffer = vec![0; 1024];
                loop {
                    let size = tap_reader.recv(&mut buffer).await.unwrap();
                    quic_writer.write_all(&buffer[..size]).await.unwrap();
                }
            })
        })
        .await
    }
    
    #[cfg(target_os = "windows")]
    Err(io::Error::new(io::ErrorKind::Other, "Windows is not supported"))
}
