use tracing::{info};
use clap::Parser;
use tokio::signal;

use tcproxy_server::{AppArguments, DefaultListener, Server};
use tcproxy_core::Result;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let args = AppArguments::parse();
    let shutdown_signal = signal::ctrl_c();
    let ip = args.get_socket_addr();
    let listener = DefaultListener::bind(ip).await?;

    Server::new(args, Box::new(listener))
        .run(shutdown_signal)
        .await?;

    info!("server stopped");
    Ok(())
}
