use tracing::{info};
use clap::Parser;
use tokio::signal;

use tcproxy::{Result, AppArguments, Server};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let args = AppArguments::parse();
    let shutdown_signal = signal::ctrl_c();
    Server::new(args)
        .run(shutdown_signal)
        .await?;

    info!("server stopped");
    Ok(())
}
