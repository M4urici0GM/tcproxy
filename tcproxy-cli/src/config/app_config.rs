use std::fs;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;
use serde::{Deserialize, Serialize};

use crate::config::{AppConfigError, AppContext};
use super::directory_resolver::DirectoryResolver;

type Result<T> = std::result::Result<T, AppConfigError>;


#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Default, Clone)]
pub struct AppConfig {
    default_context: String,
    user_token: Option<String>,
    contexts: Vec<AppContext>,
}

impl AppConfig {
    pub fn new(contexts: &[AppContext], default_context: Option<AppContext>, user_token: Option<String>) -> Self {
        Self {
            contexts: Vec::from(contexts),
            default_context: match default_context {
                Some(ctx) => ctx.name().to_string(),
                None => String::default()
            },
            user_token
        }
    }

    pub fn contexts(&self) -> &[AppContext] {
        &self.contexts
    }

    pub fn user_token(&self) -> &Option<String> {
        &self.user_token
    }
    pub fn default_context(&self) -> &str {
        &self.default_context
    }
}

pub fn load(resolver: &DirectoryResolver) -> Result<AppConfig> {
    let path = resolver.get_config_file();
    if !exists(&path) {
        create_default(&path)?;
    }

    let config = read_from_file(&path)?;
    Ok(config)
}

pub fn save_to_file(config: &AppConfig, path: &Path) -> Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .append(false)
        .create(true)
        .open(path)?;

    let self_contents = serde_yaml::to_string(&config)?;

    file.write_all(self_contents.as_bytes())?;
    file.flush()?;
    Ok(())
}

fn create_default(path: &Path) -> Result<()> {
    let default_config = AppConfig::default();
    let config_str = serde_yaml::to_string(&default_config).unwrap();
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .append(false)
        .open(path)?;

    file.write_all(config_str.as_bytes())?;
    file.flush()?;

    Ok(())
}

fn read_from_file(path: &Path) -> Result<AppConfig> {
    let file = OpenOptions::new()
        .read(true)
        .open(path)?;

    let contents = serde_yaml::from_reader::<File, AppConfig>(file)?;
    Ok(contents)
}

fn exists(path: &Path) -> bool {
    fs::metadata(path).is_ok()
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::net::IpAddr;
    use std::path::Path;
    use uuid::Uuid;
    use crate::config::{app_config::{AppConfig}, AppContext};

    #[test]
    fn should_write_to_disk() {
        let file_path = format!("./{}.yaml", Uuid::new_v4());
        let file_path = Path::new(&file_path);
        let (host, port) = create_socket_addr();

        let mut config = AppConfig::default();
        let context = AppContext::new("context1", &host, &port);

        config.set_default_context(&context);

        // Act
        AppConfig::save_to_file(&config, file_path).unwrap();

        let created_config = AppConfig::load(file_path).unwrap();

        // Assert
        assert_eq!(created_config, config);

        remove_file(file_path);
    }


    #[test]
    fn should_return_err_if_path_doesnt_exist() {
        let path = format!("~/{}.test", Uuid::new_v4());
        let file_path = Path::new(&path);
        let config = AppConfig::default();
        let result = AppConfig::save_to_file(&config, file_path);

        assert!(result.is_err());
    }

    #[test]
    fn push_context_should_return_err_if_ctx_exists() {
        let mut config = AppConfig::default();
        let (host, port) = create_socket_addr();
        let context = AppContext::new("contex1", &host, &port);

        config.push_context(&context).unwrap();

        // Act
        let result = config.push_context(&context);

        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn set_default_context_should_push_if_not_existent() {
        let mut config = AppConfig::default();
        let (host, port) = create_socket_addr();

        let context = AppContext::new("context1", &host, &port);

        // Act
        config.set_default_context(&context);

        // Assert
        assert_eq!(config.default_context_str(), context.name());
        assert_eq!(config.contexts().len(), 1);
        assert_eq!(config.contexts().get("context1").unwrap(), &context);
    }

    #[test]
    fn should_read_from_file() {
        let file_name = create_file_name();
        let config = create_default_config(&file_name);
        let path = format!("./{}", &file_name);
        let file_path = Path::new(&path);

        // Act
        let read_config = AppConfig::load(file_path).unwrap();

        // Assert
        assert_eq!(config.default_context_str(), read_config.default_context_str());
        assert_eq!(&config.contexts(), &read_config.contexts());

        remove_file(file_path);
    }

    #[test]
    pub fn ctx_exists_should_return_true() {
        // Arrange
        let mut default_config = AppConfig::default();

        let (host, port) = create_socket_addr();
        let context = AppContext::new("test-context", &host, &port);

        default_config.push_context(&context).unwrap();

        // Assert
        assert!(default_config.ctx_exists(&context));
    }

    #[test]
    pub fn setting_first_context_should_set_default_context() {
        // Arrange
        let mut default_config = AppConfig::default();
        let (host, port) = create_socket_addr();

        let context = AppContext::new("test-context", &host, &port);
        let context2 = AppContext::new("test-context2", &host, &port);

        default_config.push_context(&context).unwrap();

        // Assert
        assert_eq!(default_config.default_context_str(), context.name());
        assert_ne!(default_config.default_context_str(), context2.name());
    }


    #[test]
    pub fn should_create_file_if_doesnt_exist() {
        // Arrange
        let file_name = create_file_name();
        let path = format!("./{}", &file_name);
        let file_path = Path::new(&path);

        // Act
        let read_config = AppConfig::load(file_path);

        println!("{:?}", read_config);

        // Assert
        assert!(read_config.is_ok());

        let read_config = read_config.unwrap();
        assert!(read_config.contexts().is_empty());
        assert_eq!(read_config.default_context_str(), String::default());

        remove_file(file_path);
    }

    fn create_socket_addr() -> (String, u16) {
        let ip = IpAddr::from([127, 0, 0, 1]);

        (ip.to_string(), 80)
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
    fn remove_file(file_name: &Path) {
        std::fs::remove_file(&file_name).unwrap();
    }
}
