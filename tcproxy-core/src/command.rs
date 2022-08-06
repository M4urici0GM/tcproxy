use async_trait::async_trait;

use crate::Result;

#[async_trait]
pub trait Command: Sync + Send {
    async fn handle(&self) -> Result<()>;
}
