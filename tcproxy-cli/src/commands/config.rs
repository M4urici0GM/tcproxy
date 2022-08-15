// use async_trait::async_trait;
// use directories::{self, ProjectDirs};

// use tcproxy_core::{Result, Command};

// use crate::ConfigArgs;

// pub struct ConfigCommand<'a> {
//   args: &'a ConfigArgs
// }

// impl<'a> ConfigCommand<'a> {
//   pub fn new(args: &'a ConfigArgs) -> Self {
//     Self {
//       args
//     }
//   }
// }

// #[async_trait]
// impl<'a> Command for ConfigCommand<'a> {
//   type Output = ();

//   async fn handle(&mut self) -> Result<()> {
//     match self.args {

//     }

//     if let Some(some_dirs) = ProjectDirs::from("", "m4urici0gm", "tcproxy") {
//       let dir = some_dirs.config_dir();
//       println!("{:?}", dir);
//     }
//     Ok(())
//   }
// }
