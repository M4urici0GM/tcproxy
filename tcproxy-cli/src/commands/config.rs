use async_trait::async_trait;
use directories::{self, ProjectDirs};

use tcproxy_core::{Result, Command};

pub struct ConfigCommand;

impl ConfigCommand {
  pub fn new() -> Self {
    Self {}
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
