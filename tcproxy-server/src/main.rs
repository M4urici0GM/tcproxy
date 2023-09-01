use std::io::ErrorKind;
use std::path::PathBuf;

use clap::Parser;
use tokio::signal;
use tracing::{error, info, warn};

use tcproxy_core::config::ConfigLoader;
use tcproxy_core::tcp::{SocketListener, TcpListener};
use tcproxy_core::Result;
use tcproxy_server::managers::DefaultFeatureManager;
use tcproxy_server::{AppArguments, Server, ServerConfig};
use tokio_native_tls::native_tls::Identity;

fn get_identity_from_file(path: &PathBuf, password: &str) -> Result<Option<Identity>> {
    let file_contents = match std::fs::read(path) {
        Ok(contents) => {
            tracing::debug!("Successfully loadd certificate file");
            contents
        }
        Err(err) => match err.kind() {
            ErrorKind::NotFound => return Ok(None),
            _ => return Err(err.into()),
        },
    };

    let identity = Identity::from_pkcs12(&file_contents, password)?;
    Ok(Some(identity))
}

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

    let password = config.get_certificate_pass().to_owned().unwrap_or_default();
    let identity = match config.get_certificate_path() {
        None => None,
        Some(path) => match get_identity_from_file(&path, &password) {
            Ok(identity) => {
                tracing::debug!("successfully loaded certificate identity");
                identity
            }
            Err(err) => {
                error!("Failed when trying to load ssl certificate: {}", err);
                warn!("ignoring property certificate_path due invalid certificate file. check the path and/or the provided password");
                None
            }
        },
    };

    tracing::debug!("loaded identity: {}", identity.is_some());
    let socket_addr = config.get_socket_addr();
    let feature_manager = DefaultFeatureManager::new(config);
    let listener = TcpListener::bind(socket_addr, identity).await?;

    Server::new(feature_manager, listener)
        .run(signal::ctrl_c())
        .await?;

    info!("server stopped");
    Ok(())
}
