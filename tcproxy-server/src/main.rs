use tracing::{info};
use clap::Parser;
use tokio::signal;

use tcproxy_server::{AppArguments, Server};
use tcproxy_core::Result;
use tcproxy_core::tcp::{TcpListener, SocketListener};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let args = AppArguments::parse();
    let shutdown_signal = signal::ctrl_c();
    let ip = args.get_socket_addr();
    let listener = TcpListener::bind(ip).await?;
    let port_range = args.parse_port_range()?;
    let listen_ip = args.parse_ip()?;

    Server::new(&port_range, &listen_ip, listener)
        .run(shutdown_signal)
        .await?;

    info!("server stopped");
    Ok(())
}
