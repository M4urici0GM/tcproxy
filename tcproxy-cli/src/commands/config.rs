use async_trait::async_trait;
use directories::{self, ProjectDirs};

use tcproxy_core::{Result, Command};

use crate::ConfigArgs;

pub struct ConfigCommand {
  config_args: ConfigArgs
}

impl ConfigCommand {
  pub fn new() -> Self {
    Self {
      config_args: ConfigArgs {
        port: 16u16
      }
    }
  }
}

#[async_trait]
impl Command for ConfigCommand {
  type Output = ();

  async fn handle(&mut self) -> Result<()> {



    if let Some(some_dirs) = ProjectDirs::from("", "m4urici0gm", "tcproxy") {
      let dir = some_dirs.config_dir();
      println!("{:?}", dir);
    }
    Ok(())
  }
}
