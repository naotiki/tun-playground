use std::{future::Future, io, pin::Pin};

use tokio::io::{AsyncRead, AsyncWrite};
use ulid::Ulid;


#[async_trait::async_trait]
pub trait Server: Send + Sync {

    async fn start(&self, session_handler:AsyncSessionHandler) -> io::Result<()>;
}

pub struct AppSession {
    pub session_id: Ulid,
    pub read: Box<dyn AsyncRead + Send + Unpin>,
    pub write: Box<dyn AsyncWrite + Send + Unpin>,
}

impl AppSession{
    pub fn new(read: Box<dyn AsyncRead + Send + Unpin>, write: Box<dyn AsyncWrite + Send + Unpin>) -> Self {
       
        Self {
            session_id: Ulid::new(),
            read,
            write,
        }
    }

    
}

pub type AsyncSessionHandler = fn(session: AppSession) -> Pin<Box<dyn Future<Output = io::Result<()>> + Send>>;
