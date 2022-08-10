use async_trait::async_trait;

use crate::Result;

#[async_trait]
/// represents a issued command.
pub trait Command: Sync + Send {
    type Output;

    /// handles command request.
    async fn handle(&mut self) -> Result<Self::Output>;
}
