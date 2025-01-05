use log::{error, info};
use serde_derive::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::net::{IpAddr, SocketAddr};
use tokio::net::UdpSocket;
use tun::{Reader, Writer};

#[derive(Serialize, Deserialize)]
struct Capsule {
    data: Vec<u8>,
}
pub enum Protocol {
    Tcp,
    Udp,
    Quic,
}

pub const USING_PROTOCOL: Protocol = Protocol::Quic;

pub async fn tun_to_udp(tun: &mut Reader, udp: &UdpSocket, peer_addr: &Option<SocketAddr>) {
    let mut buffer = [0u8; 1500];
    loop {
        if let None = peer_addr {
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            continue;
        }
        match tun.read(&mut buffer) {
            Ok(n) => {
                info!("read {} bytes from TUN", n);
                let capsule = Capsule {
                    data: buffer[..n].to_vec(),
                };
                let serialized_data = bincode::serialize(&capsule).unwrap();
                udp.send_to(&serialized_data,peer_addr.unwrap()).await.unwrap();
            }
            Err(e) => {
                error!("TUN read error: {}", e);
            }
        }
    }
}

pub async fn udp_to_tun(tun: &mut Writer, udp: &UdpSocket, peer_addr: Option<&mut Option<SocketAddr>>) {
    let mut buffer = [0u8; 1500];
    loop {
        match udp.recv_from(&mut buffer).await {
            Ok((n, addr)) => {
                if  let Some(&mut ref mut a) = peer_addr {
                    *(a) = Some(addr)
                }
                let capsule: Capsule = bincode::deserialize(&buffer[..n]).unwrap();
                info!("read {} bytes from UDP", n);
                tun.write_all(capsule.data.as_slice()).unwrap();
                tun.flush().unwrap();
            }
            Err(e) => {
                error!("UDP read error: {}", e);
            }
        }
    }
}
