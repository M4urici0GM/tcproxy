
use std::{fs, net::{IpAddr, SocketAddr}, str::FromStr, ops::Range};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use serde::{Serialize, Deserialize};
use tracing::error;
use tcproxy_core::config::{Config, ConfigLoader};

use tcproxy_core::Result;

use crate::AppArguments;

pub mod env {
    pub const PORT_MIN: &str = "TCPROXY_PORT_MIN";
    pub const PORT_MAX: &str = "TCPROXY_PORT_MAX";
    pub const LISTEN_PORT: &str = "TCPROXY_LISTEN_PORT";
    pub const SERVER_FQDN: &str = "TCPROXY_SERVER_FQDN";
    pub const CONNECTIONS_PER_PROXY: &str = "TCPROXY_CONNECTIONS_PER_PROXY";
    pub const LISTEN_IP: &str = "TCPROXY_LISTEN_IP";
    pub const CONFIG_FILE: &str = "TCPROXY_CONFIG_FILE";
    pub const JWT_SECRET: &str = "TCPROXY_JWT_SECRET";
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    port_min: u16,
    port_max: u16,
    listen_ip: IpAddr,
    listen_port: u16,
    server_fqdn: String,
    max_connections_per_proxy: u16,
    jwt_secret: String,
}

// FILE
// Environment Variables
// App Arguments

impl ServerConfig {
    pub fn new(
        port_min: u16,
        port_max: u16,
        listen_ip: IpAddr,
        listen_port: u16,
        server_fqdn: &str,
        max_connections_per_proxy: u16,
        jwt_secret: &str,
    ) -> Self {
        Self {
            port_min,
            port_max,
            listen_ip,
            listen_port,
            server_fqdn: String::from(server_fqdn),
            jwt_secret: String::from(jwt_secret),
            max_connections_per_proxy,
        }
    }

    pub fn get_jwt_secret(&self) -> &str {
        &self.jwt_secret
    }

    pub fn set_jwt_secret(&mut self, secret: &str) {
        self.jwt_secret = String::from(secret);
    }

    pub fn get_socket_addr(&self) -> SocketAddr {
        SocketAddr::new(self.listen_ip, self.listen_port)
    }

    pub fn get_port_range(&self) -> Range<u16> {
        self.port_min..self.port_max
    }

    pub fn get_listen_port(&self) -> u16 {
        self.listen_port
    }

    pub fn get_listen_ip(&self) -> IpAddr {
        self.listen_ip
    }

    pub fn get_max_connections_per_proxy(&self) -> u16 {
        self.max_connections_per_proxy
    }

    pub fn get_server_fqdn(&self) -> String {
        self.server_fqdn.to_owned()
    }

    fn set_port_min(&mut self, min_port: u16) {
        self.port_min = min_port;
    }

    fn set_port_max(&mut self, max_port: u16) {
        self.port_max = max_port;
    }

    fn set_listen_port(&mut self, listen_port: u16) {
        self.listen_port = listen_port;
    }

    fn set_server_fqdn(&mut self, server_fqdn: &str) {
        self.server_fqdn = server_fqdn.to_owned();
    }

    fn set_connections_per_proxy(&mut self, connections_per_proxy: u16) {
        self.max_connections_per_proxy = connections_per_proxy;
    }

    fn set_listen_ip(&mut self, ip: IpAddr) {
        self.listen_ip = ip;
    }
}

impl Config<AppArguments> for ServerConfig {
    fn apply_env(&mut self, app_vars: &HashMap<String, String>) -> Result<()> {
        for (name, value) in app_vars {
            match name.as_str() {
                env::PORT_MIN => self.set_port_min(value.parse::<u16>()?),
                env::PORT_MAX => self.set_port_max(value.parse::<u16>()?),
                env::LISTEN_PORT => self.set_listen_port(value.parse::<u16>()?),
                env::LISTEN_IP => self.set_listen_ip(IpAddr::from_str(value)?),
                env::SERVER_FQDN => self.set_server_fqdn(value),
                env::CONNECTIONS_PER_PROXY => self.set_connections_per_proxy(value.parse::<u16>()?),
                env::JWT_SECRET => self.set_jwt_secret(value),
                _ => continue,
            }
        }

        Ok(())
    }

    fn apply_args(&mut self, args: &AppArguments) {
        if let Some(ip) = args.get_ip() {
            self.set_listen_ip(ip);
        }

        if let Some(max_conn) = args.get_max_connections_per_proxy() {
            self.set_connections_per_proxy(max_conn);
        }

        if let Some(port) = args.get_port() {
            self.set_listen_port(port);
        }

        if let Some(range) = args.get_port_range() {
            self.set_port_min(range.start);
            self.set_port_max(range.end);
        }
    }


    fn validate(&self) -> Result<()> {
        if self.port_min == 0 {
            return Err("Min port cannot be zero".into());
        }

        if self.port_min > self.port_max {
            return Err("Min port is greater than max_port".into());
        }

        Ok(())
    }
}

impl ConfigLoader<'_, AppArguments> for ServerConfig {
    fn named_environment_variables() -> HashSet<String> {
        let available_env_vars: Vec<String> = vec![
            env::PORT_MIN.to_owned(),
            env::CONNECTIONS_PER_PROXY.to_owned(),
            env::LISTEN_PORT.to_owned(),
            env::CONFIG_FILE.to_owned(),
            env::LISTEN_IP.to_owned(),
            env::SERVER_FQDN.to_owned(),
            env::PORT_MAX.to_owned()
        ];

        HashSet::from_iter(available_env_vars.iter().cloned())
    }

    fn read_from_file(path: &Path) -> Result<ServerConfig> {
        let file_contents = match fs::read_to_string(path) {
            Ok(file_contents) => file_contents,
            Err(err) => {
                error!("Failed when trying to read config file: {}", err);
                return Err(err.into());
            }
        };

        let config = serde_json::from_str::<Self>(&file_contents)?;
        Ok(config)
    }

    fn get_config_path(env_vars: &HashMap<String, String>) -> PathBuf {
        if env_vars.contains_key(env::CONFIG_FILE) {
            let config_path = env_vars
                .get(env::CONFIG_FILE)
                .unwrap()
                .to_owned();

            return PathBuf::from(&config_path);
        }

        PathBuf::from("./config.json")
    }
}


impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port_min: 15000,
            port_max: 25000,
            listen_ip: IpAddr::from_str("0.0.0.0").unwrap(),
            listen_port: 8080,
            server_fqdn: "proxy.server.local".to_owned(),
            max_connections_per_proxy: 120,
            jwt_secret: "some_secret".to_owned(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::net::IpAddr;
    use uuid::Uuid;
    use std::str::FromStr;
    use crate::{AppArguments, env, ServerConfig};
    use super::ConfigLoader;

    #[test]
    pub fn should_read_from_file() {
        // Arrange
        let file_id = Uuid::new_v4();
        let file_name = format!("{}.json", file_id);
        let args = AppArguments::default();
        let config = create_default_file(&file_name);

        let env_vars: Vec<(String, String)> = vec![(env::CONFIG_FILE.to_owned(), file_name.to_owned())];

        // Act
        let parsed_config = ServerConfig::load(&env_vars, &args).unwrap();

        // Assert
        assert_eq!(parsed_config.get_listen_ip(), config.get_listen_ip());
        assert_eq!(parsed_config.get_listen_port(), config.get_listen_port());
        assert_eq!(parsed_config.get_port_range(), config.get_port_range());
        assert_eq!(parsed_config.get_socket_addr(), config.get_socket_addr());
        assert_eq!(parsed_config.get_max_connections_per_proxy(), config.get_max_connections_per_proxy());

        remove_file(&file_name);
    }

    #[test]
    pub fn environment_variables_should_override_file() {
        // Arrange
        let file_id = Uuid::new_v4();
        let file_name = format!("{}.json", file_id);
        let args = AppArguments::default();
        let config = create_default_file(&file_name);

        let expected_port = 3337;
        let env_vars: Vec<(String, String)> = vec![
            (env::CONFIG_FILE.to_owned(), file_name.to_owned()),
            (env::LISTEN_PORT.to_owned(), expected_port.to_string())
        ];

        // Act
        let parsed_config = ServerConfig::load(&env_vars, &args).unwrap();

        // Assert
        assert_ne!(parsed_config.get_listen_port(), config.get_listen_port());
        assert_eq!(parsed_config.get_listen_port(), expected_port);

        remove_file(&file_name);
    }

    #[test]
    pub fn arguments_should_override_env_and_file() {
        // Arrange
        let file_id = Uuid::new_v4();
        let file_name = format!("{}.json", file_id);

        let expected_port = 80;
        let expected_ip = IpAddr::from_str("129.1.1.2").unwrap();
        let expected_port_range = 1111..2222;
        let expected_connections_per_proxy = 300;

        let args = AppArguments::new(
            Some(expected_port),
            Some(expected_ip),
            Some(expected_port_range.clone()),
            Some(expected_connections_per_proxy));

        let env_vars: Vec<(String, String)> = vec![
            (env::CONFIG_FILE.to_owned(), file_name.to_owned()),
            (env::LISTEN_PORT.to_owned(), 120.to_string()),
            (env::LISTEN_IP.to_owned(), "130.2.2.3".to_owned()),
            (env::PORT_MIN.to_owned(), "3333".to_owned()),
            (env::PORT_MAX.to_owned(), "4444".to_owned()),
        ];

        // Act
        let parsed_config = ServerConfig::load(&env_vars, &args).unwrap();

        // Assert
        assert_eq!(parsed_config.get_listen_port(), expected_port);
        assert_eq!(parsed_config.get_listen_ip(), expected_ip);
        assert_eq!(parsed_config.get_port_range(), expected_port_range);
        assert_eq!(parsed_config.get_max_connections_per_proxy(), expected_connections_per_proxy);

        remove_file(&file_name);
    }

    #[test]
    pub fn should_create_file_if_doesnt_exist() {
        // Arrange
        let file_id = Uuid::new_v4();
        let file_name = format!("{}.json", &file_id);
        let args = AppArguments::default();

        let env_vars: Vec<(String, String)> = vec![
            (env::CONFIG_FILE.to_owned(), file_name.to_owned())
        ];

        // Act
        let created_config = ServerConfig::load(&env_vars, &args).unwrap();

        // Assert
        assert!(std::fs::metadata(&file_name).is_ok());
        assert_eq!(
            serde_json::to_string(&created_config).unwrap(),
            std::fs::read_to_string(&file_name).unwrap());

        remove_file(&file_name);
    }

    /// Util function for removing the file after each test.
    fn remove_file(file_name: &str) {
        std::fs::remove_file(file_name).unwrap();
    }

    /// Creates default file and writes it to disk.
    fn create_default_file(file_name: &str) -> ServerConfig {
        let config = ServerConfig::new(
            10,
            20,
            IpAddr::from_str("127.0.0.1").unwrap(),
            8080,
            "proxy.server.local",
            120,
            "SOME_SECRET");

        let config_str = serde_json::to_string(&config).unwrap();
        std::fs::write(file_name, config_str).unwrap();

        config
    }
}