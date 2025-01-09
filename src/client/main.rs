use futures::{SinkExt, StreamExt};
use std::io;
use std::io::{BufWriter, Read, Write};
use std::ops::Deref;
use std::sync::Arc;
use tokio::sync::Mutex;
use tun_playground::client::quic::QuicTransport;
use tun_playground::client::tcp::TcpTransport;
use tun_playground::client::transport::Transport;
use tun_playground::client::udp::UdpTransport;
use tun_playground::protocol::{Protocol, USING_PROTOCOL};
use tun_playground::tun::TunInterface;

pub async fn create_transport(protocol: Protocol, addr: &str) -> io::Result<Box<dyn Transport>> {
    match protocol {
        Protocol::Tcp => Ok(Box::new(TcpTransport::new(addr).await?)),
        Protocol::Udp => Ok(Box::new(UdpTransport::new("0.0.0.0:0", addr).await?)),
        Protocol::Quic => Ok(Box::new(QuicTransport::new(addr).await?)),
    }
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let protocol = USING_PROTOCOL; // 動的に変更可能

    let tun = TunInterface::new("10.0.0.2".parse().unwrap());
    let (mut sink, mut stream) = tun.framed.split();
    let transport = create_transport(protocol, "127.0.0.1:8080").await?;
    let arc = Arc::new(transport);
    let cloned_arc = arc.clone();
    //copy received data to stdout using tokio::spawn
    tokio::spawn(async move {
        loop {
            let response = arc.receive().await.unwrap();
            // print response utf-8
            print!("server:{}", String::from_utf8_lossy(&response));
            sink.send(response).await.unwrap();
        
        }
    });
    // read from stdin and send to server
    loop {
        tokio::select! {
            Some(packet)=stream.next()=>{
                let pkt: Vec<u8> = packet?;
                cloned_arc.send(&pkt).await?;
            }
        }
        /*let mut buffer = vec![0; 1024];
        let n = io::stdin().read(&mut buffer)?;
        cloned_arc.send(&buffer[..n]).await?;*/
    }

    /*loop {
        let mut buffer = vec![0; 1024];
        let n = io::stdin().read(&mut buffer)?;
        transport.send(&buffer[..n]).await?;
        let response = transport.receive().await?;
        // print response utf-8
        print!("server:{}", String::from_utf8_lossy(&response));
    }*/

    /*transport.send(b"Hello, Server!").await?;
    let response = transport.receive().await?;
    println!("Response: {:?}", response);*/

    Ok(())
}
