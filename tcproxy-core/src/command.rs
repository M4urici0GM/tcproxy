use async_trait::async_trait;


/// represents a issued command.
pub trait Command: Sync + Send {
    type Output;

    /// handles command request.
    fn handle(&mut self) -> Self::Output;
}

#[async_trait]
pub trait AsyncCommand: Sync + Send {
    type Output;

    async fn handle(&mut self) -> Self::Output;
}

