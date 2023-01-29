use std::fmt::{Display, Formatter};
use tcproxy_core::Error;
use crate::config::{AppConfigError, AppContext};

#[derive(Debug)]
pub enum AppContextError {
    DoesntExist(String),
    AlreadyExists(AppContext),
    ConfigError(Error),
    Other(Error),
}

impl std::error::Error for AppContextError {}

impl Display for AppContextError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            AppContextError::DoesntExist(ctx_name) => format!("context {} doesn't exists.", ctx_name),
            AppContextError::AlreadyExists(ctx) => format!("context {} with ip {} already exists", ctx.name(), ctx.host()),
            AppContextError::ConfigError(err) => format!("config error: {}", err),
            AppContextError::Other(err) => format!("unexpected error: {}", err),
        };

        write!(f, "{}", msg)
    }
}

impl From<String> for AppContextError {
    fn from(msg: String) -> Self {
        AppContextError::Other(msg.into())
    }
}

impl From<&str> for AppContextError {
    fn from(msg: &str) -> Self {
        AppContextError::Other(msg.into())
    }
}

impl From<AppConfigError> for AppContextError {
    fn from(value: AppConfigError) -> Self {
        Self::ConfigError(value.into())
    }
}