use async_trait::async_trait;
use mockall::automock;
use tcproxy_core::auth::User;
use uuid::Uuid;

pub enum AccountManagerError {
    NotFound,
    Other(tcproxy_core::Error),
}

pub struct DatabaseManager {
} 

impl DatabaseManager {
    pub fn new() -> Self {
        Self {

        }
    }
}

#[automock]
#[async_trait]
pub trait UserManager: Sync + Send {
    async fn find_account_by_id(&self, account_id: &Uuid) -> Result<User, AccountManagerError>;
    async fn find_user_by_email(&self, email: &str) -> Result<User, AccountManagerError>;
}

pub struct DefaultAccountManager {
}

impl DefaultAccountManager {
    pub fn new() -> Self {
        Self {
        }
    }
}

#[async_trait]
impl UserManager for DefaultAccountManager {
    async fn find_account_by_id(&self, account_id: &Uuid) -> Result<User, AccountManagerError> {
        let maybe_user = None;
        let user_details = match maybe_user {
            Some(user) => user,
            None => return Err(AccountManagerError::NotFound),
        };

        Ok(user_details)
    }

    async fn find_user_by_email(&self, email: &str) -> Result<User, AccountManagerError> {
        let maybe_user = None;
        let user_details = match maybe_user {
            Some(user) => user,
            None => return Err(AccountManagerError::NotFound),
        };

        Ok(user_details)
    }
}

