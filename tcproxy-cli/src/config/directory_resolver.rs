use std::path::{PathBuf, Path};
use directories::ProjectDirs;

use tcproxy_core::Result;

#[derive(Debug, Clone)]
pub struct DirectoryResolver {
    path: PathBuf,
    file_name: String,
}

const ORGANIZATION_NAME: &'static str = "m4urici0gm";
const APPLICATION_NAME: &'static str = "tcproxy";
const QUALIFIER: &'static str = "";
const FILE_NAME: &'static str = "config.yaml";

impl DirectoryResolver {
    pub fn new(path: &Path, name: &str) -> Self {
        Self {
            path: path.to_owned(),
            file_name: String::from(name),
        }
    }

    pub fn get_config_file(&self) -> PathBuf {
        let mut base_path = self.path.clone();
        base_path.push(&self.file_name);
    
        base_path
    }
}

pub fn load() -> Result<DirectoryResolver> {
    let config_dir = get_config_dir()?.config_dir();
    if !config_dir.exists() {
        std::fs::create_dir_all(config_dir)?;
    }

    let path_buff = PathBuf::from(&config_dir);

    Ok(DirectoryResolver::new(&path_buff, FILE_NAME))
}


fn get_config_dir() -> Result<ProjectDirs> {
    match ProjectDirs::from(&QUALIFIER, &ORGANIZATION_NAME, &APPLICATION_NAME) {
        Some(dir) => Ok(dir),
        None => Err("Couldnt access config folder".into()),
    }
}