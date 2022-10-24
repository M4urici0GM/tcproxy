use std::sync::Arc;
use clap::Parser;
use tokio::signal;
use tracing::{info, error};

use tcproxy_core::tcp::{SocketListener, TcpListener};
use tcproxy_core::Result;
use tcproxy_server::{AppArguments, Server, ServerConfig};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let env_vars: Vec<(String, String)> = std::env::vars().collect();
    let args = AppArguments::parse();
    
    let config = match ServerConfig::load(&env_vars, &args) {
        Ok(config) => config,
        Err(err) => {
            error!("Failed when parsing config. Check your file/environment variables: {}", err);
            panic!("Cannot start with invalid config!");
        }
    };

    let config = Arc::new(config);
    let listener = TcpListener::bind(config.get_socket_addr()).await?;
    Server::new(config, listener)
        .run(signal::ctrl_c())
        .await?;

    info!("server stopped");
    Ok(())
}
