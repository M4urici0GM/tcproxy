pub mod app_config;
mod app_config_error;
mod app_context;
mod app_context_error;
pub mod context_manager;
pub mod directory_resolver;

pub use app_config_error::AppConfigError;
pub use app_context::*;
pub use app_context_error::AppContextError;

use std::sync::{Arc, Mutex, MutexGuard};
use tracing::info;

use self::{context_manager::ContextManager, directory_resolver::DirectoryResolver};
use crate::config::app_config::AppConfig;
use tcproxy_core::{auth::token_handler::AuthToken, Result};

#[derive(Debug, Clone)]
pub struct AuthManager {
    current_token: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Config {
    contexts: Arc<Mutex<ContextManager>>,
    auth: Arc<Mutex<AuthManager>>,
}

impl AuthManager {
    pub fn new(token: Option<String>) -> Self {
        Self {
            current_token: token,
        }
    }

    pub fn current_token(&self) -> &Option<String> {
        &self.current_token
    }

    pub fn set_current_token(&mut self, value: Option<AuthToken>) {
        self.current_token = value.map(|t| t.get().to_string());
    }
}

impl Config {
    pub fn new(contexts: &ContextManager, auth: &AuthManager) -> Self {
        Self {
            contexts: Arc::new(Mutex::new(contexts.clone())),
            auth: Arc::new(Mutex::new(auth.clone())),
        }
    }

    pub fn lock_context_manager(&self) -> Result<MutexGuard<'_, ContextManager>> {
        Ok(self.contexts.lock().unwrap()) // TODO: fix me
    }

    pub fn lock_auth_manager(&self) -> Result<MutexGuard<'_, AuthManager>> {
        Ok(self.auth.lock().unwrap()) // TODO: fix me
    }
}

pub fn save_to_disk(config: &Config, directory_resolver: &DirectoryResolver) -> Result<()> {
    let path = directory_resolver.get_config_file();
    info!("trying to load config file from {:?}", path);
    let context_manager = config.lock_context_manager()?;
    let auth_manager = config.lock_auth_manager()?;

    let app_config = AppConfig::new(
        context_manager.contexts_arr(),
        context_manager.default_context(),
        auth_manager.current_token().clone(),
    );

    app_config::save_to_file(&app_config, &path)?;
    Ok(())
}

pub fn load(directory_resolver: &DirectoryResolver) -> Result<Config> {
    let config_file = app_config::load(directory_resolver)?;

    // initialize managers
    let context_manager =
        ContextManager::new(config_file.default_context(), config_file.contexts());

    let auth = AuthManager::new(config_file.user_token().clone());
    let config = Config::new(&context_manager, &auth);
    Ok(config)
}
