use clap::Parser;
use tcproxy_cli::{App, ClientArgs};
use tcproxy_core::Result;
use tokio::signal;

#[tokio::main]
async fn main() -> Result<()> {
    let args = ClientArgs::parse();
    App::new(args).start(signal::ctrl_c()).await?;

    Ok(())
}
