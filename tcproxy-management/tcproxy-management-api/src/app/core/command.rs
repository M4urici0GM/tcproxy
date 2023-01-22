use async_trait::async_trait;

pub trait Command {
    type Output;
}

#[async_trait]
pub trait CommandHandler<T>
    where T : Command
{
    async fn execute_cmd(&self, cmd: T) -> T::Output;
}