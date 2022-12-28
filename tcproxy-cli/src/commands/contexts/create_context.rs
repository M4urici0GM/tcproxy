use std::path::PathBuf;

use tcproxy_core::{Command, Result};
use crate::config::{AppConfig, AppContext};
use crate::CreateContextArgs;

use super::DirectoryResolver;

pub mod env {
    pub const CONFIG_NAME: &str = "config.yaml";
}

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

    fn get_full_config_path(&self) -> Result<PathBuf> {
        let config_path = self.dir_resolver.get_config_folder()?;
        let mut path_buf = PathBuf::from(&config_path);

        path_buf.push(env::CONFIG_NAME);

        Ok(path_buf)
    }
}

impl Command for CreateContextCommand {
    type Output = tcproxy_core::Result<()>;

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
    #[test]
    fn should_create_file_if_doesnt_exist() {

    }
}