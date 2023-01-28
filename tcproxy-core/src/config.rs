use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::Result;

pub trait Config<T> {
    fn apply_env(&mut self, env: &HashMap<String, String>) -> Result<()>;
    fn apply_args(&mut self, args: &T);
    fn validate(&self) -> Result<()>;
}

pub trait ConfigLoader<'a, T, Args>
    where
        T: 'a + Config<Args>,
        T: Deserialize<'a>,
        T: Serialize,
        T: Default
{
    /// function that return available environment names.
    fn named_environment_variables() -> HashSet<String>;

    /// reads config file from disk.
    fn read_from_file(path: &Path) -> Result<T>;

    /// gets where config should be read from.
    fn get_config_path(environment_variables: &HashMap<String, String>) -> PathBuf;

    /// loads config from environment variables
    fn load(env_vars: &[(String, String)], args: &Args) -> Result<T> {
        let parsed_env_vars = Self::parse_environment_variables(env_vars);
        let config_path = Self::get_config_path(&parsed_env_vars);

        if !Self::file_exists(&config_path) {
            debug!("Config file doesnt exist. Creating default...");
            Self::create_default(&config_path)?;
        }

        let mut config = Self::read_from_file(&config_path)?;

        config.apply_env(&parsed_env_vars)?;
        config.apply_args(args);
        config.validate()?;

        Ok(config)
    }

    fn parse_environment_variables(env_vars: &[(String, String)]) -> HashMap<String, String> {
        let mut hash_map = HashMap::<String, String>::new();
        let available_env_vars = Self::named_environment_variables();

        for (key, value) in env_vars {
            if available_env_vars.contains(key) {
                hash_map.insert(key.to_owned(), value.to_owned());
            }
        }

        hash_map
    }

    fn file_exists(file_path: &Path) -> bool {
        fs::metadata(file_path).is_ok()
    }

    fn create_default(file_path: &Path) -> Result<()> {
        let config = T::default();
        let config_str = serde_json::to_string(&config)?;

        fs::write(file_path, config_str)?;
        Ok(())
    }
}