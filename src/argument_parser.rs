use clap::{command, Parser};
use std::io::{Read, Write};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, ToSocketAddrs};
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::io;
use tun::{BoxError, Configuration, Layer, ToAddress};
use crate::protocol::{tun_to_udp, udp_to_tun};

#[derive(clap::Parser, Debug)]
#[command(name = "tunquic", version, about, author, long_about = None)]
pub struct Argument {
    #[command(subcommand)]
    command: Commands,
    #[arg(short, long, required = true)]
    tun_addr: String,
    #[arg(long, short = 'v', global = true)]
    verbose: bool,
    #[arg(long, global = true)]
    debug: bool,
}

impl Argument {
    pub async fn exec(&self) -> Result<(), BoxError> {
        if self.debug {
            println!("{:?}", self);
        }

        let tun_ipaddr = self.tun_addr.to_address().unwrap();
        println!("ip:{}",tun_ipaddr);
        // create TUN device
        let mut config = Configuration::default();
        config
            .address(tun_ipaddr)
            .netmask((255, 255, 255, 0))
            .destination((10, 0, 0, 1))
            .up();

        #[cfg(target_os = "linux")]
        config.platform_config(|config| {
            // requiring root privilege to acquire complete functions
            config.ensure_root_privileges(true);
        });

        let dev = tun::create(&config)?;
        let ( mut dev_read,mut dev_write) = dev.split();
        match &self.command {
            Commands::Server { listen } => {
                let udp_socket = UdpSocket::bind(listen).await.unwrap();
                let r = Arc::new(udp_socket);
                let s = r.clone();

                let mut peer_addr:IpAddr= IpAddr::V4(Ipv4Addr::UNSPECIFIED);
                let pipe = tokio::spawn(async move {
                    /*let mut buf = [0; 4096];
                    loop {
                        if peer_addr.is_unspecified() {
                            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                            continue;
                        }
                        let amount = dev_read.read(&mut buf).unwrap();
                        println!("{:?} bytes received from tun", amount);
                        s.send_to(&buf[0..amount], s.peer_addr().unwrap())
                            .await
                            .unwrap();
                    }*/
                    tun_to_udp(&mut dev_read, &s, &peer_addr).await;
                });
                /*let mut buf = [0; 4096];
                loop {
                    let (len, addr) = r.recv_from(&mut buf).await.unwrap();
                    peer_addr = addr.ip().clone();
                    println!("{:?} bytes received from {:?}", len, addr);
                    dev_write.write(&buf[..len]).unwrap();
                }*/
                udp_to_tun(&mut dev_write, &r, Some(&mut peer_addr)).await;
            }
            Commands::Client { host } => {
                let server_addrs = host.to_socket_addrs().unwrap().next().unwrap();
                let addr = server_addrs.to_address().unwrap();
                let udp_socket = (match server_addrs {
                    SocketAddr::V4(_) => {
                        UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)).await as io::Result<UdpSocket>
                    }
                    SocketAddr::V6(_) => {
                        UdpSocket::bind((Ipv6Addr::UNSPECIFIED, 0)).await as io::Result<UdpSocket>
                    }
                })
                .unwrap();

                let r = Arc::new(udp_socket);
                let s = r.clone();
                let pipe = tokio::spawn(async move {
                    /*let mut buf = [0; 4096];
                    loop {
                        let amount = dev_read.read(&mut buf).unwrap();
                        println!("{:?} bytes received from tun", amount);
                        s.send_to(&buf[0..amount], server_addrs).await.unwrap();
                    }*/
                    tun_to_udp(&mut dev_read, &s, &addr).await;
                });
                /*let mut buf = [0; 4096];
                loop {
                    let (len, addr) = r.recv_from(&mut buf).await.unwrap();
                    println!("{:?} bytes received from {:?}", len, addr);
                    dev_write.write(&buf[..len]).unwrap();
                }*/
                udp_to_tun(&mut dev_write, &r,None).await;
            }
        }
        println!("end:exec");
        Ok(())
    }
}

#[derive(clap::Subcommand, Debug)]
pub enum Commands {
    #[command(alias = "s")]
    Server {
        /// address to listen
        #[arg(short, long, default_value = "0.0.0.0:12345")]
        listen: String,
    },
    #[command(alias = "c")]
    Client {
        /// host tp connect
        #[arg()]
        host: String,
    },
}

pub fn parse_args() -> Argument {
    return Argument::parse();
}
