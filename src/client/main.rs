use futures::{SinkExt, StreamExt};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::io;
use tun_playground::client::quic::QuicTransport;
use tun_playground::client::tcp::TcpTransport;
use tun_playground::client::transport::Transport;
use tun_playground::protocol::{Protocol, USING_PROTOCOL};
use tun_playground::tun::TunInterface;

pub async fn create_transport(protocol: Protocol, addr: &str) -> io::Result<Box<dyn Transport>> {
    match protocol {
        Protocol::Tcp => Ok(Box::new(TcpTransport::new(addr).await?)),
        Protocol::Quic => Ok(Box::new(QuicTransport::new(addr).await?)),
    }
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let protocol = USING_PROTOCOL; // 動的に変更可能

    let tun = TunInterface::new("10.0.0.2".parse().unwrap());
    let (mut tun_sink, mut tun_stream) = tun.framed.split();
    let transport = create_transport(protocol, "127.0.0.1:8080").await?;
    let (mut recv, mut send) = transport.split();
    
    //copy received data to stdout using tokio::spawn
    tokio::spawn(async move {
        loop {
            
            let mut buffer = vec![0; 1024];
            let n = recv.read(&mut buffer).await.unwrap();
            buffer.truncate(n);
            // print response utf-8
            print!("server:{}", String::from_utf8_lossy(&buffer));
            tun_sink.send(buffer).await.unwrap();
        
        }
    });
    // read from stdin and send to server
    loop {
        tokio::select! {
            Some(packet)=tun_stream.next()=>{
                let pkt: Vec<u8> = packet?;
                send.write_all(&pkt).await.unwrap();
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

}
