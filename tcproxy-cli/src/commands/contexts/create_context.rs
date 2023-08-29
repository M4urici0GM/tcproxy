use crate::config::{AppContext, AppContextError, Config};
use crate::server_addr::ServerAddrType;
use crate::CreateContextArgs;
use tcproxy_core::Command;

pub struct CreateContextCommand {
    args: CreateContextArgs,
    config: Config,
}

impl CreateContextCommand {
    pub fn new(args: &CreateContextArgs, config: &Config) -> Self {
        Self {
            args: args.clone(),
            config: config.clone(),
        }
    }
}

impl Command for CreateContextCommand {
    type Output = Result<(), AppContextError>;

    fn handle(&mut self) -> Self::Output {
        let context_addr = self.args.host();
        let context = AppContext::from_addr(self.args.name(), context_addr);

        // temporary! need to implement a dns resolver
        // TODO: implement dns resolving module.
        if context_addr.addr_type() != ServerAddrType::IpAddr {
            return Err(AppContextError::ValidationError(
                "Cannot accept DNS hosts.".to_string(),
            ));
        }

        push_context(&self.config, &context)?;

        println!("created context {}", self.args.name());
        Ok(())
    }
}

fn push_context(config: &Config, context: &AppContext) -> tcproxy_core::Result<()> {
    let mut context_manager = config.lock_context_manager()?;
    context_manager.push_context(context)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    // #[test]
    // fn should_return_err_when_host_is_not_ip() {
    //     let mut dir_resolver = MockDirectoryResolver::new();
    //     let file_path = create_random_file_path();

    //     {
    //         let file_path = file_path.clone();
    //         dir_resolver.expect_get_config_file().returning(move || { Ok(file_path.clone()) });
    //     }

    //     let server_addr = ServerAddr::new("tcp.someserveraddress.io", &8080).unwrap();
    //     let context_args = CreateContextArgs::new("test-name", &server_addr);
    //     let mut command = CreateContextCommand::new(&context_args, dir_resolver);

    //     // Act
    //     let result = command.handle();

    //     // Assert
    //     assert!(result.is_err());
    //     assert!(is_type!(result.unwrap_err(), AppContextError::ValidationError(_)));

    // }

    // #[test]
    // fn should_create_file_if_doesnt_exist() {
    //     // Arrange
    //     let mut dir_resolver = MockDirectoryResolver::new();
    //     let file_path = create_random_file_path();

    //     {
    //         let file_path = file_path.clone();
    //         dir_resolver.expect_get_config_file().returning(move || { Ok(file_path.clone()) });
    //     }

    //     let server_addr = ServerAddr::new("127.0.0.1", &8080).unwrap();
    //     let context_args = CreateContextArgs::new("test-name", &server_addr);
    //     let mut command = CreateContextCommand::new(&context_args, dir_resolver);

    //     // Act
    //     let result = command.handle();

    //     // Assert
    //     assert!(result.is_ok());
    //     assert!(fs::metadata(&file_path).is_ok());

    //     remove_file(&file_path);
    // }

    // #[test]
    // fn should_return_err_when_ctx_already_exists() {
    //     // Arrange
    //     let mut dir_resolver = MockDirectoryResolver::new();
    //     let file_path = create_random_file_path();

    //     {
    //         let file_path = file_path.clone();
    //         dir_resolver.expect_get_config_file().returning(move || { Ok(file_path.clone()) });
    //     }

    //     let server_addr = ServerAddr::new("127.0.0.1", &8080).unwrap();
    //     let context_args = CreateContextArgs::new("test-name", &server_addr);
    //     let mut command = CreateContextCommand::new(&context_args, dir_resolver);

    //     // Act
    //     let result = command.handle();
    //     let result1 = command.handle();

    //     // Assert
    //     assert!(result.is_ok());
    //     assert!(fs::metadata(&file_path).is_ok());

    //     assert!(result1.is_err());
    //     assert!(is_type!(result1.unwrap_err(), AppContextError::AlreadyExists(_)));

    //     remove_file(&file_path);
    // }

    // fn create_random_file_path() -> PathBuf {
    //     let file_name = Uuid::new_v4();
    //     let full_path = format!("./{}.yaml", &file_name);

    //     PathBuf::from(&full_path)
    // }

    // /// Util function for removing the file after each test.
    // fn remove_file(path: &Path) {
    //     std::fs::remove_file(path).unwrap();
    // }
}
