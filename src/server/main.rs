use futures::{FutureExt, SinkExt, StreamExt};
use packet::{ether, ip, Builder};
use std::io;
use tokio::io::AsyncWriteExt;
use tokio_util::codec::{FramedRead, FramedWrite};
use tokio_util::io::ReaderStream;
use tun::Tun;
use tun_playground::protocol::{Frame, Protocol, TunnelCodec, USING_PROTOCOL};
use tun_playground::server::asynctap::AsyncTap;
use tun_playground::server::quic::QuicServer;
use tun_playground::server::server::{AppSession, Server};
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
    #[cfg(not(target_os = "windows"))]
    {
        server
            .start(|session: AppSession| handle_session(session).boxed())
            .await?;
    }

    Ok(())
}

async fn handle_session(session: AppSession) -> io::Result<()> {
    let tap = AsyncTap::new()?;
    let (tap_reader, mut tap_writer) = tokio::io::split(tap);
    let mut tap_reader = ReaderStream::new(tap_reader);

    let (mut frame_read, mut frame_send) = (
        FramedRead::new(session.read, TunnelCodec),
        FramedWrite::new(session.write, TunnelCodec),
    );

    loop {
        tokio::select! {
            Some(quic_to_tun) = frame_read.next() => {
                match quic_to_tun {
                    Ok(frame) => {
                        if let Frame::IPv4(data) = frame {
                            let eth=ether::Builder::default().destination(
                                "00:00:00:00:00:00".parse().unwrap()
                            ).unwrap().source(
                                "00:00:00:00:00:00".parse().unwrap()
                            ).unwrap().protocol(
                                ether::Protocol::Ipv4
                            ).unwrap().payload(
                                &data
                            ).unwrap();

                            let eth_bytes = eth.build().unwrap();

                            tap_writer.write_all(&eth_bytes).await
                                .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
                        }
                    }
                    Err(e) => return Err(io::Error::new(io::ErrorKind::Other, e.to_string()))
                }
            }
            Some(Ok(pkt)) = tap_reader.next() => {
                if let Ok(pkt) = ether::Packet::new(pkt) {
                    let frame = Frame::IPv4(pkt.as_ref().to_vec());
                    frame_send.send(frame).await
                        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
                }
            }
            else => {

            }
        }
    }
}
