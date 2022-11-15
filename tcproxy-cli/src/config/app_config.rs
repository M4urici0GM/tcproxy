use std::borrow::Borrow;
use std::fmt::{Display, Formatter, write};
use std::fs;
use resolve_path::PathResolveExt;
use serde::{Deserialize, Serialize};
use tracing::error;

use tcproxy_core::{Error};

use crate::config::{AppContext, AppContextError};

type Result<T> = std::result::Result<T, AppConfigError>;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct AppConfig {
    default_context: String,
    contexts: Vec<AppContext>,
}

#[derive(Debug)]
pub enum AppConfigError {
    DeserializationError(Error),
    SerializationError(Error),
    WriteError(Error),
    Other(Error)
}

impl AppConfig {
    pub fn load(path: &str) -> Result<Self> {
        let actual_path = AppConfig::canonicalize_path(&path)?;
        if !AppConfig::exists(&actual_path) {
            AppConfig::create_default(&actual_path)?;
        }

        let config = AppConfig::read_from_file(&path)?;
        Ok(config)
    }

    pub fn default_context(&self) -> &str {
        &self.default_context
    }

    pub fn contexts(&self) -> &[AppContext] {
        &self.contexts
    }


    pub fn ctx_exists(&self, context: &AppContext) -> bool {
        self.contexts
            .iter()
            .any(|ctx| *ctx == *context)
    }

    pub fn set_default_context(&mut self, context: &AppContext) -> bool {
        if !self.ctx_exists(&context) {
            self.contexts.push(context.clone());
            return false;
        }

        self.default_context = context.name().to_owned();
        true
    }

    pub fn has_default_context(&self) -> bool {
        self.default_context != String::default()
    }

    pub fn push_context(&mut self, context: &AppContext) -> std::result::Result<(), AppContextError> {
        if self.ctx_exists(context) {
            return Err(AppContextError::AlreadyExists(context.clone()))
        }

        self.contexts.push(context.clone());
        if !self.has_default_context() {
            self.set_default_context(context);
        }

        Ok(())
    }

    fn canonicalize_path(path: &str) -> Result<String> {
        let resolved_path = path.resolve();
        let path_buf = resolved_path.display().to_string();

        Ok(String::from(path_buf))
    }

    fn create_default(path: &str) -> Result<()> {
        let default_config = AppConfig::default();
        let config_str = serde_yaml::to_string(&default_config).unwrap();

        match fs::write(&path, &config_str) {
            Ok(_) => Ok(()),
            Err(err) => Err(AppConfigError::WriteError(err.into())),
        }
    }

    fn read_from_file(path: &str) -> Result<Self> {
        let file_contents = match fs::read_to_string(&path) {
            Ok(contents) => contents,
            Err(err) => {
                error!("Failed when reading config from file: {}", err);
                return Err(AppConfigError::Other(err.into()));
            }
        };

        match serde_yaml::from_str::<Self>(&file_contents) {
            Ok(cfg) => Ok(cfg),
            Err(err) => Err(AppConfigError::DeserializationError(err.into()))
        }
    }

    fn exists(path: &str) -> bool {
        fs::metadata(&path).is_ok()
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            contexts: Vec::default(),
            default_context: String::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::net::{IpAddr, SocketAddr};
    use uuid::Uuid;
    use crate::config::{AppConfig, AppContext};

    #[test]
    fn should_read_from_file() {
        let file_name = create_file_name();
        let config = create_default_config(&file_name);

        // Act
        let read_config = AppConfig::load(&file_name).unwrap();

        // Assert
        assert_eq!(config.default_context(), read_config.default_context());
        assert_eq!(&config.contexts()[..], &read_config.contexts()[..]);

        remove_file(&file_name);
    }

    #[test]
    pub fn ctx_exists_should_return_true() {
        // Arrange
        let mut default_config = AppConfig::default();

        let target_ip = IpAddr::from([127, 0, 0, 1]);
        let socket_addr = SocketAddr::new(target_ip, 8080);
        let context = AppContext::new("test-context", &socket_addr);

        default_config.push_context(&context).unwrap();

        // Assert
        assert!(default_config.ctx_exists(&context));
    }

    #[test]
    pub fn setting_first_context_should_set_default_context() {
        // Arrange
        let mut default_config = AppConfig::default();

        let target_ip = IpAddr::from([127, 0, 0, 1]);
        let socket_addr = SocketAddr::new(target_ip, 8080);
        let context = AppContext::new("test-context", &socket_addr);
        let context2 = AppContext::new("test-context2", &socket_addr);

        default_config.push_context(&context).unwrap();

        // Assert
        assert_eq!(default_config.default_context(), context.name());
        assert_ne!(default_config.default_context(), context2.name());
    }


    #[test]
    pub fn should_create_file_if_doesnt_exist() {
        // Arrange
        let file_name = create_file_name();

        // Act
        let read_config = AppConfig::load(&file_name);

        println!("{:?}", read_config);

        // Assert
        assert!(read_config.is_ok());

        let read_config = read_config.unwrap();
        assert_eq!(&read_config.contexts()[..], &vec![]);
        assert_eq!(read_config.default_context(), String::default());
    }

    fn create_file_name() -> String {
        let file_id = Uuid::new_v4();
        format!("{}.yaml",  file_id)
    }

    /// Creates empty config and writes it to disk.
    fn create_default_config(path: &str) -> AppConfig {
        let empty_config = AppConfig::default();
        let yaml_config = serde_yaml::to_string(&empty_config).unwrap();

        let _ = fs::write(&path, &yaml_config);

        empty_config
    }

    /// Util function for removing file after test.
    fn remove_file(file_name: &str) {
        std::fs::remove_file(&file_name).unwrap();
    }
}

impl From<String> for AppConfigError {
    fn from(msg: String) -> Self {
        AppConfigError::Other(msg.into())
    }
}

impl From<&str> for AppConfigError {
    fn from(msg: &str) -> Self {
        AppConfigError::Other(msg.into())
    }
}


impl std::error::Error for AppConfigError {}

impl Display for AppConfigError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            AppConfigError::DeserializationError(err) => format!("Error when deserializing object: {}", err),
            AppConfigError::SerializationError(err) => format!("Error when serializing object: {}", err),
            AppConfigError::WriteError(err) => format!("Failed when writing file to disk: {}", err),
            AppConfigError::Other(err) => format!("Unexpected error! {}", err),
        };

        write!(f, "{}", msg)
    }
}