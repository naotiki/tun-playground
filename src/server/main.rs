use futures::{FutureExt, SinkExt, StreamExt};
use netns_rs::NetNs;
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
use tun_playground::tun::TunInterface;

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
    let ns = NetNs::get("test-ns").unwrap();
    let tun = ns
        .run(|_| TunInterface::new("10.0.0.254".parse().unwrap(), "10.0.0.1".parse().unwrap()))
        .unwrap();

    let (mut tun_sink, mut tun_stream) = tun.device.into_framed().split();

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
                        tun_sink.send(data).await
                            .unwrap();
                    }
                }
                Err(e) => {}
            }
        }
        Some(Ok(pkt)) = tun_stream.next() => {
            if let Ok(pkt) = ether::Packet::new(pkt) {
                let frame = Frame::IPv4(pkt.as_ref().to_vec());
                frame_send.send(frame).await.unwrap();
            }
        }
        }
    }
}
