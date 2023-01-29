use tcproxy_core::Command;
use crate::commands::contexts::DirectoryResolver;
use crate::config::{AppConfig, AppContextError};
use crate::SetDefaultContextArgs;

pub struct SetDefaultContextCommand {
    args: SetDefaultContextArgs,
    dir_resolver: Box<dyn DirectoryResolver + 'static>
}

impl SetDefaultContextCommand {
    pub fn new<T>(args: &SetDefaultContextArgs, dir_resolver: T) -> Self
        where T : DirectoryResolver + 'static
    {
        Self {
            args: args.clone(),
            dir_resolver: Box::new(dir_resolver),
        }
    }
}

impl Command for SetDefaultContextCommand {
    type Output = Result<(), AppContextError>;

    fn handle(&mut self) -> Self::Output {
        let config_path = self.dir_resolver.get_config_file()?;
        let mut config = AppConfig::load(&config_path)?;

        let context = match config.get_context(self.args.name()) {
            Some(ctx) => ctx,
            None => {
                return Err(AppContextError::DoesntExist(self.args.name().to_string()));
            }
        };

        config.set_default_context(&context);
        AppConfig::save_to_file(&config, &config_path)?;

        println!("successfully set {} as default context", context);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};
    use uuid::Uuid;
    use tcproxy_core::{Command, is_type};
    use crate::commands::contexts::*;
    use crate::config::{AppConfig, AppContext, AppContextError};
    use crate::SetDefaultContextArgs;

    #[test]
    pub fn should_return_err_when_context_doesnt_exist() {
        // Arrange
        let full_path = format!("./{}.yaml", &Uuid::new_v4());
        let file_path = PathBuf::from(&full_path);

        let mut dir_resolver = MockDirectoryResolver::new();
        // create new scope for cloning the variable.
        {
            let file_path = file_path.clone();
            dir_resolver.expect_get_config_file().returning(move || { Ok(file_path.clone()) });
        }

        let command_args = SetDefaultContextArgs::new("test-ctx");
        let mut command = SetDefaultContextCommand::new(&command_args, dir_resolver);

        // Act
        let result = command.handle();

        // Assert
        assert!(result.is_err());
        assert!(is_type!(result.unwrap_err(), AppContextError::DoesntExist(_)));

        std::fs::remove_file(&file_path).unwrap();
    }

    #[test]
    pub fn should_set_default_context() {
        // Arrange
        let full_path = format!("./{}.yaml", &Uuid::new_v4());
        let file_path = PathBuf::from(&full_path);

        let mut dir_resolver = MockDirectoryResolver::new();
        // create new scope for cloning the variable.
        {
            let file_path = file_path.clone();
            dir_resolver.expect_get_config_file().returning(move || { Ok(file_path.clone()) });
        }

        let mut default_config = AppConfig::default();
        default_config.push_context(&AppContext::new("test-ctx" , "127.0.0.1", &8080)).unwrap();
        default_config.push_context(&AppContext::new("test-ctx2" , "127.0.0.1", &8081)).unwrap();

        AppConfig::save_to_file(&default_config, &file_path).unwrap();

        let command_args = SetDefaultContextArgs::new("test-ctx2");
        let mut command = SetDefaultContextCommand::new(&command_args, dir_resolver);

        // Act
        let result = command.handle();

        // Assert
        assert!(result.is_ok());

        std::fs::remove_file(&file_path).unwrap();
    }
}