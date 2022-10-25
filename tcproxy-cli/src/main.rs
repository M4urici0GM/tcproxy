use clap::Parser;
use tcproxy_cli::{App, ClientArgs};
use tcproxy_core::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let args = ClientArgs::parse();
    App::new(args).start().await?;

    Ok(())
}
