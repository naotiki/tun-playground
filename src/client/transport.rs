
use async_trait::async_trait;
use tokio::io::{AsyncRead, AsyncWrite};

#[async_trait]
pub trait Transport: Send + Sync {

    fn split(self: Box<Self>) -> (Box<dyn AsyncRead+Send + Unpin>, Box<dyn AsyncWrite + Send + Unpin>);
}
