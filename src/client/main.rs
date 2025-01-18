use futures::{SinkExt, StreamExt};
use packet::{ip, AsPacket, Packet};
use std::collections::HashMap;
use std::io;
use std::net::Ipv4Addr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_util::bytes::BytesMut;
use tokio_util::codec::{Decoder, Encoder, FramedRead, FramedWrite};
use tun_playground::client::nat::{NATTable, Network};
use tun_playground::client::quic::QuicTransport;
use tun_playground::client::tcp::TcpTransport;
use tun_playground::client::transport::Transport;
use tun_playground::protocol::{Frame, Protocol, TunnelCodec, USING_PROTOCOL};
use tun_playground::tun::TunInterface;

pub async fn create_transport(protocol: Protocol, addr: &str) -> io::Result<Box<dyn Transport>> {
    match protocol {
        Protocol::Tcp => Ok(Box::new(TcpTransport::new(addr).await?)),
        Protocol::Quic => Ok(Box::new(QuicTransport::new(addr).await?)),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut nat_table = NATTable::new();

    nat_table.add_entry(
        Network::new(Ipv4Addr::new(10, 1, 0, 0), 24),
        Network::new(Ipv4Addr::new(10, 0, 0, 0), 24),
    );

    let protocol = USING_PROTOCOL; // 動的に変更可能

    let tun = TunInterface::new("10.1.0.2".parse().unwrap());
    let (mut tun_sink, mut tun_stream) = tun.device.into_framed().split();

    let transport = create_transport(protocol, "127.0.0.1:8080").await?;
    let (recv, send) = transport.split();

    let mut frame_read = FramedRead::new(recv, TunnelCodec);
    let mut frame_send = FramedWrite::new(send, TunnelCodec);
    //copy received data to stdout using tokio::spawn
    loop{
        tokio::select! {
            Some(quic_to_tun) = frame_read.next()=>{
                match quic_to_tun {
                    Ok(frame)=>{
                        println!("received frame: {:#?}", frame);
                    match frame {
                        Frame::IPv4(data) => match ip::Packet::new(data) {
                            Ok(ip::Packet::V4(mut pkt)) => {
                                let pkt = pkt
                                    .set_destination(nat_table.reverse(pkt.destination()))
                                    .unwrap();
                                let pkt_bytes = pkt.as_ref();
                                tun_sink.send(pkt_bytes.to_vec()).await.unwrap();
                            }
                            _ => {
                                eprintln!("something wrong");
                            }
                        },
                        _ => {}
                    };
                    }
                    Err(e)=>{
                        eprintln!("error: {:#?}", e);
                    }
                }
            }
            Some(tun_to_quic) = tun_stream.next() => {
                let pkt: Vec<u8> = tun_to_quic.unwrap();
                match ip::Packet::new(pkt) {
                    Ok(ip::Packet::V4(mut pkt)) => {
                        let pkt = pkt.set_destination(nat_table.convert(pkt.destination()))?;

                        let pkt_bytes = pkt.as_ref();
                        let frame = Frame::IPv4(pkt_bytes.to_vec());
                        frame_send.send(frame).await.unwrap();
                    }
                    Ok(ip::Packet::V6(_pkt)) => {
                        eprintln!("IPv6 packet is not supported");
                    }
                    Err(e) => {
                        eprintln!("error:{:#?}", e);
                    }
                }
            }
        }
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

