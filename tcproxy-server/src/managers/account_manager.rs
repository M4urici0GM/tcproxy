use std::sync::Arc;
use async_trait::async_trait;
use mockall::automock;
use mongodb::{Collection, Database};
use mongodb::bson::{doc, Uuid};
use mongodb::error::Error;
use tcproxy_core::auth::User;

pub enum AccountManagerError {
    NotFound,
    Other(tcproxy_core::Error)
}

impl From<mongodb::error::Error> for AccountManagerError {
    fn from(value: Error) -> Self {
        Self::Other(value.into())
    }
}

#[automock]
#[async_trait]
pub trait UserManager: Sync + Send {
    async fn find_account_by_id(&self, account_id: &Uuid) -> Result<User, AccountManagerError>;
    async fn find_user_by_email(&self, email: &str) -> Result<User, AccountManagerError>;
}

pub struct DefaultAccountManager {
    collection: Collection<User>,
}

impl DefaultAccountManager {
    pub fn new(database: &Arc<Database>) -> Self {
        Self {
            collection: database.collection("users"),
        }
    }
}

#[async_trait]
impl UserManager for DefaultAccountManager {
    async fn find_account_by_id(&self, account_id: &Uuid) -> Result<User, AccountManagerError> {
        let query = doc!{ "_id": account_id };
        let user_details = self.collection
            .find_one(query, None)
            .await?
            .unwrap();

        Ok(user_details)
    }

    async fn find_user_by_email(&self, email: &str) -> Result<User, AccountManagerError> {
        let query = doc!{ "email": email };
        let user_details = self.collection
            .find_one(query, None)
            .await?
            .unwrap();

        Ok(user_details)
    }
}