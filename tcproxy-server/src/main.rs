use clap::Parser;
use tokio::signal;
use tracing::{error, info};

use tcproxy_core::config::ConfigLoader;
use tcproxy_core::tcp::{SocketListener, TcpListener};
use tcproxy_core::Result;
use tcproxy_server::managers::DefaultFeatureManager;
use tcproxy_server::{AppArguments, Server, ServerConfig};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let env_vars: Vec<(String, String)> = std::env::vars().collect();
    let args = AppArguments::parse();

    let config = match ServerConfig::load(&env_vars, &args) {
        Ok(config) => config,
        Err(err) => {
            error!(
                "Failed when parsing config. Check your file/environment variables: {}",
                err
            );
            panic!("Cannot start with invalid config!");
        }
    };

    let socket_addr = config.get_socket_addr();
    let feature_manager = DefaultFeatureManager::new(config);
    let listener = TcpListener::bind(socket_addr, None).await?;

    Server::new(feature_manager, listener)
        .run(signal::ctrl_c())
        .await?;

    info!("server stopped");
    Ok(())
}
