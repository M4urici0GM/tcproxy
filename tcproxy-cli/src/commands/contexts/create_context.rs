use std::fs;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::net::{IpAddr};
use std::path::{Path, PathBuf};
use serde::{Serialize, Deserialize};
use directories::{self, ProjectDirs};

use tcproxy_core::{Command, Result};

use crate::CreateContextArgs;

pub struct CreateContextCommand {
    args: CreateContextArgs
}

impl CreateContextCommand {
    pub fn new(args: &CreateContextArgs) -> Self {
        Self {
            args: args.clone(),
        }
    }

    fn get_config_dir(&self) -> Option<ProjectDirs> {
        ProjectDirs::from("", "m4urici0gm", "tcproxy")
    }
}

fn file_exists(path: &Path) -> bool {
    fs::metadata(path).is_ok()
}

fn open_or_create(path: &Path) -> Result<File> {
    let file_exists_val = file_exists(path);
    let mut options = OpenOptions::new();

    println!("{} {}", file_exists_val, path.to_str().unwrap());

    let parent = path.parent().unwrap();
    if !parent.exists() {
        println!("trying to create {} folder", parent.to_str().unwrap());
        fs::create_dir_all(parent)?;
    }

    let mut file = options
        .write(true)
        .read(true)
        .create(true)
        .open(path)?;

    let empty_config = AppConfig {
        default_context: String::default(),
        contexts: vec![],
    };

    let yaml = serde_yaml::to_string(&empty_config)?;
    file.write_all(yaml.as_bytes())?;

    Ok(file)
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct ContextConfig {
    name: String,
    ip: IpAddr
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct AppConfig {
    default_context: String,
    contexts: Vec<ContextConfig>,
}


impl Command for CreateContextCommand {
    type Output = tcproxy_core::Result<()>;

    fn handle(&mut self) -> Self::Output {
        let dir = match self.get_config_dir() {
            Some(dir) => dir,
            None => return Err("Couldnt access config folder".into()),
        };

        let config_path = dir.config_dir();
        let mut path_buf = PathBuf::from(config_path);
        path_buf.push("config.yaml");

        let mut file = open_or_create(path_buf.as_path())?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let mut config = serde_yaml::from_str::<AppConfig>(&contents)?;

        let name= &self.args.name;
        if config.contexts.iter().any(|cfg| { cfg.name == self.args.name && cfg.ip == self.args.host }) {
            return Err("".into());
        }

        if config.default_context == String::default() {
            config.default_context = name.to_owned();
            config.contexts.push(ContextConfig {
                ip: self.args.host,
                name: name.to_owned()
            });
        } else {
            config.contexts.push(ContextConfig {
                ip: self.args.host,
                name: name.to_owned()
            });
        }

        println!("TEST");
        let stream = serde_yaml::to_string(&config)?;
        drop(file);

        let mut file = OpenOptions::new().read(true).write(true).open(path_buf.as_path())?;
        file.write_all(stream.as_bytes())?;
        Ok(())
    }
}
