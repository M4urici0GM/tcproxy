mod create_context;
mod list_contexts;

use mockall::automock;

use tcproxy_core::Result;

#[automock]
pub trait DirectoryResolver {
    fn get_user_folder(&self) -> Result<String>;
    fn get_config_folder(&self) -> Result<String>;
}

pub use create_context::CreateContextCommand;
pub use list_contexts::ListContextsCommand;

