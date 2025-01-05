use std::cell::RefCell;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::rc::Rc;
use std::time::Duration;
use tokio::io::{copy, AsyncReadExt, AsyncWriteExt};
use tokio::time::sleep;
use tun_playground::quic_client::QuicClientBackend;
use tun_playground::quic_server::{QuicServerBackend, QuicServerSession};
use tun_playground::tunnel_backend::{
    ClientConnectionConfig, TransportTunnelClientBackend, TransportTunnelServerBackend,
};

#[tokio::main]
async fn main() {
    /*let args = argument_parser::parse_args();
    args.exec().await.unwrap();*/
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 8080);
    let server = tokio::spawn(async move {
        let server = QuicServerBackend::serve(addr).await;
        //let mut sessions: Vec<Rc<RefCell<QuicServerSession>>> = Vec::new();
        let session = server.accept().await.unwrap();
        println!("accepted");
        //let mut session = Rc::new(RefCell::new(session));
        //sessions.push(session);

        //let sess = session.get_mut();
        let (mut send, mut recv) = (session.send, session.recv);
        copy(&mut recv, &mut send).await.unwrap();
        /*tokio::spawn(async move {
            /*let buf = &mut [0u8; 4096];
            loop {
                let Some(n) = recv.read(buf).await.unwrap() else { break; };
                if n == 0 {
                    break;
                }
                println!("server:{}", std::str::from_utf8(&buf[..n]).unwrap());
                send.write("server:".as_ref()).await.unwrap();
                send.write_all(&buf[..n]).await.unwrap();
                send.flush().await.unwrap();
            }*/
        });*/

        println!("server shutdown");
    });
    println!("server started");
    println!("sleeping for 3 seconds");
    sleep(Duration::from_secs(3)).await;

    let mut client = QuicClientBackend::connect(ClientConnectionConfig {
        server_addr: addr,
        server_name: "localhost".to_string(),
    })
    .await;
    println!("client connected");
    tokio::spawn(async move {
        println!("client send loop");
        let buf = &mut [0u8; 4096];
        loop {
            let n = tokio::io::stdin().read(buf).await.unwrap();
            if n == 0 {
                break;
            }
            println!("stdin:{}", std::str::from_utf8(&buf[..n]).unwrap());
            client.send.write_all(&buf[..n]).await.unwrap();
            client.send.flush().await.unwrap();
        }
        println!("client send loop end");
    });
    tokio::spawn(async move {
        println!("client recv loop");
        let buf = &mut [0u8; 4096];
        loop {
            let Some(n) = client.recv.read(buf).await.unwrap() else {
                break;
            };
            if n == 0 {
                break;
            }
            println!("client:{}", std::str::from_utf8(&buf[..n]).unwrap());
        }
        println!("client recv loop end");
    })
    .await
    .unwrap();
}
