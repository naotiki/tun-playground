use log::{error, info};
use serde_derive::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::net::IpAddr;
use std::ptr;
use tokio::net::UdpSocket;
use tun::{Reader, ToAddress, Writer};

#[derive(Serialize, Deserialize)]
struct Capsule {
    data: Vec<u8>,
}

pub async fn tun_to_udp(tun: &mut Reader, udp: &UdpSocket, peer_addr: &IpAddr) {
    let mut buffer = [0u8; 1500];
    loop {
        if peer_addr.is_unspecified() {
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
                udp.send(&serialized_data).await.unwrap();
            }
            Err(e) => {
                error!("TUN read error: {}", e);
            }
        }
    }
}

pub async fn udp_to_tun(tun: &mut Writer, udp: &UdpSocket, peer_addr: Option<&mut IpAddr>) {
    let mut buffer = [0u8; 1500];
    loop {
        match udp.recv_from(&mut buffer).await {
            Ok((n, addr)) => {
                if  let Some(&mut ref mut a) = peer_addr {
                    *(a) = addr.to_address().unwrap();
                }
                let capsule: Capsule = bincode::deserialize(&buffer[..n]).unwrap();
                info!("read {} bytes from UDP", n);
                tun.write_all(capsule.data.as_slice()).unwrap();
            }
            Err(e) => {
                error!("UDP read error: {}", e);
            }
        }
    }
}
