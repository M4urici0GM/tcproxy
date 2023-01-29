use std::path::PathBuf;

use tcproxy_core::{Command};
use crate::config::{AppConfig, AppContext, AppContextError};
use crate::CreateContextArgs;

use super::DirectoryResolver;

pub struct CreateContextCommand {
    args: CreateContextArgs,
    dir_resolver: Box<dyn DirectoryResolver + 'static>,
}

impl CreateContextCommand {
    pub fn new<T>(args: &CreateContextArgs, dir_resolver: T) -> Self where T : DirectoryResolver + 'static {
        Self {
            args: args.clone(),
            dir_resolver: Box::new(dir_resolver)
        }
    }

    fn get_full_config_path(&self) -> Result<PathBuf, AppContextError> {
        let config_path = match self.dir_resolver.get_config_folder() {
            Ok(path) => path,
            Err(err) => {
                return Err(AppContextError::Other(err));
            }
        };

        Ok(PathBuf::from(&config_path))
    }
}

impl Command for CreateContextCommand {
    type Output = Result<(), AppContextError>;

    fn handle(&mut self) -> Self::Output {
        let config_path = self.get_full_config_path()?;

        let context_addr = self.args.host();
        let context = AppContext::new(self.args.name(), context_addr.host(), context_addr.port());
        let mut config = AppConfig::load(&config_path)?;

        config.push_context(&context)?;

        AppConfig::save_to_file(&config, &config_path)?;
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;
    use uuid::Uuid;
    use tcproxy_core::{Command, is_type};
    use crate::commands::contexts::*;
    use crate::config::AppContextError;
    use crate::CreateContextArgs;
    use crate::server_addr::ServerAddr;

    #[test]
    fn should_create_file_if_doesnt_exist() {
        // Arrange
        let mut dir_resolver = MockDirectoryResolver::new();
        let file_path = create_random_file_path();

        {
            let file_path = file_path.clone();
            dir_resolver.expect_get_config_folder().returning(move || { Ok(file_path.clone()) });
        }

        let server_addr = ServerAddr::new("127.0.0.1", &8080).unwrap();
        let context_args = CreateContextArgs::new("test-name", &server_addr);
        let mut command = CreateContextCommand::new(&context_args, dir_resolver);

        // Act
        let result = command.handle();

        // Assert
        assert!(result.is_ok());
        assert!(fs::metadata(&file_path).is_ok());

        remove_file(&file_path);
    }

    #[test]
    fn should_return_err_when_ctx_already_exists() {
        // Arrange
        let mut dir_resolver = MockDirectoryResolver::new();
        let file_path = create_random_file_path();

        {
            let file_path = file_path.clone();
            dir_resolver.expect_get_config_folder().returning(move || { Ok(file_path.clone()) });
        }

        let server_addr = ServerAddr::new("127.0.0.1", &8080).unwrap();
        let context_args = CreateContextArgs::new("test-name", &server_addr);
        let mut command = CreateContextCommand::new(&context_args, dir_resolver);

        // Act
        let result = command.handle();
        let result1 = command.handle();

        // Assert
        assert!(result.is_ok());
        assert!(fs::metadata(&file_path).is_ok());

        assert!(result1.is_err());
        assert!(is_type!(result1.unwrap_err(), AppContextError::AlreadyExists(_)));

        remove_file(&file_path);
    }

    fn create_random_file_path() -> PathBuf {
        let file_name = Uuid::new_v4();
        let full_path = format!("./{}.yaml", &file_name);

        PathBuf::from(&full_path)
    }

    /// Util function for removing the file after each test.
    fn remove_file(path: &Path) {
        std::fs::remove_file(path).unwrap();
    }
}