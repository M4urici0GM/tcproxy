use std::fmt::{Display, Formatter};
use directories::{self, ProjectDirs};
use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::net::{IpAddr, SocketAddr};
use std::path::{Path, PathBuf};
use tracing::error;
use tracing_subscriber::fmt::format;

use tcproxy_core::{Command, Error, Result};
use crate::config::{AppConfig, AppContext};

use crate::CreateContextArgs;

pub mod env {
    pub const CONFIG_NAME: &str = "config.yaml";
}

pub struct CreateContextCommand {
    args: CreateContextArgs,
}

impl CreateContextCommand {
    pub fn new(args: &CreateContextArgs) -> Self {
        Self { args: args.clone() }
    }

    fn get_config_dir(&self) -> Option<ProjectDirs> {
        ProjectDirs::from("", "m4urici0gm", "tcproxy")
    }
}

impl Command for CreateContextCommand {
    type Output = tcproxy_core::Result<()>;

    fn handle(&mut self) -> Self::Output {
        let dir = match self.get_config_dir() {
            Some(dir) => dir,
            None => return Err("Couldnt access config folder".into()),
        };

        let path = dir.config_dir();
        let mut path_buf = PathBuf::from(&path);
        path_buf.push("config.yaml");

        let final_path = match path_buf.to_str() {
            Some(path) => path,
            None => return Err(format!("couldnt access {:?}", path_buf).into()),
        };

        let context = AppContext::new(&self.args.name, &self.args.host);
        let mut config = AppConfig::load(&final_path)?;

        config.push_context(&context)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {

}