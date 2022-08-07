use clap::Parser;
use tcproxy_cli::{ClientArgs, App};
use tcproxy_core::Result;
use tokio::signal;

#[tokio::main]
async fn main() -> Result<()> {
    let args = ClientArgs::parse();
    App::new(args)
        .start()
        .await?;

    Ok(())
}
