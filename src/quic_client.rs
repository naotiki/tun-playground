use crate::tunnel_backend::{ClientConnectionConfig, TransportTunnelClientBackend};
use quinn::crypto::rustls::QuicClientConfig;
use quinn::{ClientConfig, ConnectError, Connection, ConnectionError, Endpoint};
use rustls::pki_types::{CertificateDer, ServerName, UnixTime};
use std::error::Error;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream};
use std::sync::Arc;
use tokio::io::{copy, copy_bidirectional_with_sizes, duplex, AsyncRead, AsyncWrite};

fn configure_client() -> Result<Endpoint, Box<dyn Error + Send + Sync + 'static>> {
    let mut endpoint = Endpoint::client(SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0))?;

    endpoint.set_default_client_config(ClientConfig::new(Arc::new(QuicClientConfig::try_from(
        rustls::ClientConfig::builder()
            .dangerous()
            .with_custom_certificate_verifier(SkipServerVerification::new())
            .with_no_client_auth(),
    )?)));
    Ok(endpoint)
}

pub struct QuicClientBackend {
    client_endpoint: Arc<Endpoint>,
    connection: Connection,
    pub recv: quinn::RecvStream,
    pub send: quinn::SendStream,
}

impl TransportTunnelClientBackend for QuicClientBackend {
    // create new session
    async fn connect(client_config: ClientConnectionConfig) -> QuicClientBackend {
        let client_endpoint = Arc::new(configure_client().unwrap());
        let connection = client_endpoint
            .connect(client_config.server_addr, &*client_config.server_name)
            .unwrap()
            .await
            .unwrap();
        println!("[client] connected: addr={}", connection.remote_address());
        let (send, recv) = connection.open_bi().await.unwrap();
        QuicClientBackend {
            client_endpoint,
            connection,
            recv,
            send,
        }
    }

    async fn close_session(self) {
        self.connection.close(0u32.into(), b"done");
        self.client_endpoint.wait_idle().await;
    }
}

/// Dummy certificate verifier that treats any certificate as valid.
/// NOTE, such verification is vulnerable to MITM attacks, but convenient for testing.
#[derive(Debug)]
pub(crate) struct SkipServerVerification(Arc<rustls::crypto::CryptoProvider>);

impl SkipServerVerification {
    fn new() -> Arc<Self> {
        Arc::new(Self(Arc::new(rustls::crypto::ring::default_provider())))
    }
}

impl rustls::client::danger::ServerCertVerifier for SkipServerVerification {
    fn verify_server_cert(
        &self,
        _end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp: &[u8],
        _now: UnixTime,
    ) -> Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::danger::ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        rustls::crypto::verify_tls12_signature(
            message,
            cert,
            dss,
            &self.0.signature_verification_algorithms,
        )
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        rustls::crypto::verify_tls13_signature(
            message,
            cert,
            dss,
            &self.0.signature_verification_algorithms,
        )
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        self.0.signature_verification_algorithms.supported_schemes()
    }
}
