
use std::io;
use async_trait::async_trait;

#[async_trait]
pub trait Transport: Send + Sync {
    async fn send(&self, data: &[u8]) -> io::Result<()>;
    async fn receive(&self) -> io::Result<Vec<u8>>;
}
