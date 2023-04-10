use tcproxy_core::Command;
use crate::config::{AppContextError, self, Config, AppContext};
use crate::SetDefaultContextArgs;

pub struct SetDefaultContextCommand {
    args: SetDefaultContextArgs,
    config: Config,
}

impl SetDefaultContextCommand {
    pub fn new(args: &SetDefaultContextArgs, config: &Config) -> Self {
        Self {
            args: args.clone(),
            config: config.clone(),
        }
    }
}

impl Command for SetDefaultContextCommand {
    type Output = Result<(), AppContextError>;

    fn handle(&mut self) -> Self::Output {
        let context = set_default_context(&self.config, self.args.name())?;
        self.config.save_to_disk()?;

        println!("successfully set {} as default context", context);

        Ok(())
    }
}

fn set_default_context(config: &Config, name: &str) -> Result<AppContext, AppContextError> {
    let mut context_manager = config.lock_context_manager()?;
    let context = match context_manager.get_context(name) {
        Some(ctx) => ctx,
        None => {
            return Err(AppContextError::DoesntExist(name.to_string()));
        }
    };

    context_manager.set_default_context(&context);
    Ok(context)
}

#[cfg(test)]
mod tests {
    // use std::path::{PathBuf};
    // use std::sync::Arc;
    // use uuid::Uuid;
    // use tcproxy_core::{Command, is_type};
    // use crate::commands::contexts::*;
    // use crate::config::context_manager::ContextManager;
    // use crate::config::directory_resolver::{DirectoryResolver};
    // use crate::config::{AppContext, AppContextError, ConfigGuard, Config, self};
    // use crate::SetDefaultContextArgs;

    // #[test]
    // pub fn should_return_err_when_context_doesnt_exist() {
    //     // Arrange
    //     let path = "./";
    //     let file_name = format!("{}.yaml", &Uuid::new_v4());
    //     let file_path = PathBuf::from(&file_name);
    //     let directory_resolver = DirectoryResolver::new(&file_path, &file_name);

    //     let config = config::load(&directory_resolver).unwrap();

        
    //     let command_args = SetDefaultContextArgs::new("test-ctx");
    //     let mut command = SetDefaultContextCommand::new(&command_args, &directory_resolver);

    //     // Act
    //     let result = command.handle();

    //     // Assert
    //     assert!(result.is_err());
    //     assert!(is_type!(result.unwrap_err(), AppContextError::DoesntExist(_)));

    //     std::fs::remove_file(&file_path).unwrap();
    // }

    // #[test]
    // pub fn should_set_default_context() {
    //     // Arrange
    //     let path = "./";
    //     let file_name = format!("{}.yaml", &Uuid::new_v4());
    //     let file_path = PathBuf::from(&file_name);
    //     let directory_resolver = Arc::new(DirectoryResolver::new(&file_path, &file_name));
    //     let mut context_manager = ContextManager::default();

    //     context_manager.push_context(&AppContext::new("test-ctx" , "127.0.0.1", &8080)).unwrap();
    //     context_manager.push_context(&AppContext::new("test-ctx2" , "127.0.0.1", &8081)).unwrap();

    //     let config = Config::new(&context_manager, directory_resolver);
    //     let guard = ConfigGuard::new(config, directory_resolver.clone());

    //     let command_args = SetDefaultContextArgs::new("test-ctx2");
    //     let mut command = SetDefaultContextCommand::new(&command_args, &directory_resolver);

    //     // Act
    //     let result = command.handle();

    //     // Assert
    //     assert!(result.is_ok());

    //     std::fs::remove_file(&file_path).unwrap();
    // }
}