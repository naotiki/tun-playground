use std::io;

#[async_trait::async_trait]
pub trait Server: Send + Sync {

    async fn start(&self,) -> io::Result<()>;
}