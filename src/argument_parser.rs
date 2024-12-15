use clap::{command, Parser};
use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr, ToSocketAddrs};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt};
use tokio::net::UdpSocket;
use tokio::{io, join};
use tokio::sync::mpsc;
use tun::ToAddress;

#[derive(clap::Parser, Debug)]
#[command(name = "tunquic", version, about, author, long_about = None)]
pub struct Argument {
    #[command(subcommand)]
    command: Commands,
    #[arg(short, long,  required = true)]
    tun_addr: String,
    #[arg(long, short = 'v', global = true)]
    verbose: bool,
    #[arg(long, global = true)]
    debug: bool,
}

impl Argument {
    pub async fn exec(&self) {
        if self.debug {
            println!("{:?}", self);
        }

        let tun_ipaddr = self.tun_addr.to_address().unwrap();

        // create TUN device
        let mut config = tun::Configuration::default();
        config
            .address(tun_ipaddr)
            .netmask((255, 255, 255, 0))
            .destination((10, 0, 0, 1))
            .tun_name("tunquick0")
            .up();
        
        #[cfg(target_os = "linux")]
        config.platform_config(|config| {
            // requiring root privilege to acquire complete functions
            config.ensure_root_privileges(true);
        });
        
        let (mut dev_write, mut dev_read) = tun::create_as_async(&config).unwrap().split().unwrap();

        async fn pipe<R, W>(read: &mut R, write: &mut W)
        where
            R: io::AsyncRead + Unpin,
            W: io::AsyncWrite + Unpin,
        {
            let mut buf = [0; 4096];
            println!("pipe:created");
            loop {
                let amount = read.read(&mut buf).await.unwrap();
                write.write(&buf[0..amount]).await.unwrap();
            }
        }
        match &self.command {
            Commands::Server { listen } => {
                let udp_socket = UdpSocket::bind(listen).await.unwrap();
                let r = Arc::new(udp_socket);
                let s = r.clone();
                
                
                let pipe =
                    tokio::spawn(async move {
                        let mut buf = [0; 4096];
                        loop {
                            let amount= dev_read.read(&mut buf).await.unwrap();
                            println!("{:?} bytes received from tun", amount);   
                            s.send_to(&buf[0..amount], s.peer_addr().unwrap()).await.unwrap();
                        }
                    });
                let mut buf = [0; 4096];
                loop {
                    let (len, addr) = r.recv_from(&mut buf).await.unwrap();
                    println!("{:?} bytes received from {:?}", len, addr);
                    dev_write.write(&buf[..len]).await.unwrap();
                }

            }
            Commands::Client { host } => {
                let server_addrs = host.to_socket_addrs().unwrap().next().unwrap();
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


                let pipe =
                    tokio::spawn(async move {
                        let mut buf = [0; 4096];
                        loop {
                            let amount= dev_read.read(&mut buf).await.unwrap();
                            println!("{:?} bytes received from tun", amount);
                            s.send_to(&buf[0..amount], server_addrs).await.unwrap();
                        }
                    });
                let mut buf = [0; 4096];
                loop {
                    let (len, addr) = r.recv_from(&mut buf).await.unwrap();
                    println!("{:?} bytes received from {:?}", len, addr);
                    dev_write.write(&buf[..len]).await.unwrap();
                }
            }
        }
        println!("end:exec");
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
