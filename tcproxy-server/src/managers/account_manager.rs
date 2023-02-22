use async_trait::async_trait;
use mockall::automock;
use tcproxy_core::auth::AccountDetails;

pub enum AccountManagerError {
    UserNotFound,
    Other(tcproxy_core::Error)
}

#[automock]
#[async_trait]
pub trait AccountManager: Sync + Send {
    async fn find_account_by_account_id(&self, account_id: &str) -> Result<AccountDetails, AccountManagerError>;
}