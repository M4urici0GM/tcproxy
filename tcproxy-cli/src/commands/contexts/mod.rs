mod create_context;
mod list_contexts;
mod set_default_context;

use std::path::PathBuf;
use mockall::automock;

use tcproxy_core::Result;

#[automock]
pub trait DirectoryResolver: Send + Sync {
    fn get_config_folder(&self) -> Result<PathBuf>;
    fn get_config_file(&self) -> Result<PathBuf>;
}

pub use create_context::CreateContextCommand;
pub use list_contexts::ListContextsCommand;
pub use set_default_context::SetDefaultContextCommand;

