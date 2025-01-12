use bollard::network::CreateNetworkOptions;
use bollard::Docker;
use futures::{SinkExt, StreamExt};
use tun_playground::tun::TunInterface;
use std::collections::HashMap;
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

    server
        .start(|session| {
            Box::pin(async move {
                let tun = TunInterface::new("10.0.0.1".parse().unwrap());
                let (mut sink, mut stream) = tun.framed.split();
                
                println!("Session ID: {}", session.session_id);
                let mut read = session.read;
                let mut write = session.write;
                tokio::spawn(async move {
                    let mut buffer = vec![0; 1024];
                    loop {
                        let response = read.read(&mut buffer).await.unwrap();
                        // print response utf-8
                        //print!("server:{}", String::from_utf8_lossy(&buffer[..response]));
                        sink.send(buffer[..response].to_vec()).await.unwrap();
                    }
                });
                
                loop {
                    tokio::select! {
                        Some(packet)=stream.next()=>{
                            let pkt: Vec<u8> = packet?;
                            write.write_all(&pkt).await.unwrap();
                        }
                    }
                }
                Ok(())
            })
        })
        .await
}
