
use clap::Parser;
use mongodb::Client;
use tokio::signal;
use tracing::{info, error};

use tcproxy_core::tcp::{SocketListener, TcpListener};
use tcproxy_core::Result;
use tcproxy_core::config::ConfigLoader;
use tcproxy_server::{AppArguments, Server, ServerConfig};
use tcproxy_server::managers::DefaultFeatureManager;

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

    let db_client = Client::with_uri_str(config.get_connection_string()).await?;
    let socket_addr = config.get_socket_addr();
    let feature_manager = DefaultFeatureManager::new(config);
    let listener = TcpListener::bind(socket_addr).await?;

    Server::new(feature_manager, listener, db_client)
        .run(signal::ctrl_c())
        .await?;

    info!("server stopped");
    Ok(())
}
