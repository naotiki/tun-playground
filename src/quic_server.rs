use crate::tunnel_backend::{TransportTunnelServerBackend, TransportTunnelSessionBackend};
use quinn::{Endpoint, ServerConfig};
use rustls::pki_types::{CertificateDer, PrivatePkcs8KeyDer};
use std::error::Error;
use std::net::SocketAddr;
use std::sync::Arc;

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
fn make_server_endpoint(
    bind_addr: SocketAddr,
) -> Result<(Endpoint, CertificateDer<'static>), Box<dyn Error + Send + Sync + 'static>> {
    let (server_config, server_cert) = configure_server()?;
    let endpoint = Endpoint::server(server_config, bind_addr)?;
    Ok((endpoint, server_cert))
}

pub struct QuicServerBackend {
    server_endpoint: Endpoint,
}

impl TransportTunnelServerBackend for QuicServerBackend {
    async fn serve(listen_addr: SocketAddr) -> Self {
        let (endpoint, _cert_der) = make_server_endpoint(listen_addr).unwrap();
        QuicServerBackend {
            server_endpoint: endpoint,
        }
    }

    async fn accept(&self) -> Result<QuicServerSession, ()> {
        let Some(conn) = self.server_endpoint.accept().await else {
            return Err(());
        };
        let Ok(connection) = conn.await else {
            return Err(());
        };
        let (send, recv) = connection.open_bi().await.map_err(|_e| ())?;
        let session = QuicServerSession {
            conn: connection,
            send,
            recv,
        };
        Ok(session)
    }

    async fn shutdown(self) {
        self.server_endpoint.wait_idle().await;
    }
}

pub struct QuicServerSession {
    conn: quinn::Connection,
    pub send: quinn::SendStream,
    pub recv: quinn::RecvStream,
}

impl TransportTunnelSessionBackend for QuicServerSession {
    async fn close_session(self) {
        self.conn.close(0u32.into(), b"done");
    }

    /*async fn pipe_write(&mut self, write: impl AsyncWrite) {
        copy(&mut self.recv, &mut Box::pin(write)).await.unwrap();
    }

    async fn pipe_read(&mut self, read: impl AsyncRead) {
        copy(&mut Box::pin(read), &mut Box::pin(&mut self.send))
            .await
            .unwrap();
    }*/
}
