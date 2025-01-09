use std::error::Error;
use std::io;
use std::net::SocketAddr;
use quinn::{Endpoint, ServerConfig};
use std::sync::Arc;
use rustls::pki_types::{CertificateDer, PrivatePkcs8KeyDer};
use crate::server::server::Server;


pub struct QuicServer {
    address: String,
}

impl QuicServer {
    pub fn new(address: &str) -> Self {
        Self {
            address: address.to_string(),
        }
    }

    async fn handle_connection(conn: quinn::Connection) -> io::Result<()> {
        let (mut send, mut recv) = conn.accept_bi().await?;
        
        let mut buffer = vec![0; 1024];
        loop {
            let Some(n) = recv.read(&mut buffer).await? else { break; };
            if n == 0 {
                break;
            }
            println!("Received: {:?}", &buffer[..n]);
            send.write_all(&buffer[..n]).await?; // Echo back the data
        }
        Ok(())
    }
}

#[async_trait::async_trait]
impl Server for QuicServer {
    async fn start(&self) -> io::Result<()> {
        let (endpoint, _server_cert) = make_server_endpoint(self.address.parse().unwrap()).unwrap();
        println!("QUIC server listening on {}", self.address);

        while let Some(conn) = endpoint.accept().await {
            let new_connection = conn.await?;
            println!("New QUIC connection: {:?}", new_connection.remote_address());

            tokio::spawn(async move {
                if let Err(e) = QuicServer::handle_connection(new_connection).await {
                    eprintln!("Connection failed: {}", e);
                }
            });
        }
        Ok(())
    }
}


pub fn make_server_endpoint(
    bind_addr: SocketAddr,
) -> Result<(Endpoint, CertificateDer<'static>), Box<dyn Error + Send + Sync + 'static>> {
    let (server_config, server_cert) = configure_server()?;
    let endpoint = Endpoint::server(server_config, bind_addr)?;
    Ok((endpoint, server_cert))
}
/// Returns default server configuration along with its certificate.
fn configure_server(
) -> Result<(ServerConfig, CertificateDer<'static>), Box<dyn Error + Send + Sync + 'static>> {
    let cert = rcgen::generate_simple_self_signed(vec!["localhost".into()]).unwrap();
    let cert_der = CertificateDer::from(cert.cert);
    let priv_key = PrivatePkcs8KeyDer::from(cert.key_pair.serialize_der());

    let mut server_config =
        ServerConfig::with_single_cert(vec![cert_der.clone()], priv_key.into())?;
    
    let transport_config = Arc::get_mut(&mut server_config.transport).unwrap();
    transport_config.max_concurrent_uni_streams(0_u8.into());

    Ok((server_config, cert_der))
}