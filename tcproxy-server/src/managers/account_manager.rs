use std::sync::Arc;
use async_trait::async_trait;
use mockall::automock;
use mongodb::{Collection, Database};
use mongodb::bson::{doc, Uuid};
use mongodb::error::Error;
use tcproxy_core::auth::{Account, AccountDetailsDto, UserDetails};

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
pub trait AccountManager: Sync + Send {
    async fn find_account_by_id(&self, account_id: &Uuid) -> Result<AccountDetailsDto, AccountManagerError>;
}

pub struct DefaultAccountManager {
    user_collection: Collection<UserDetails>,
    collection: Collection<Account>,
}

impl DefaultAccountManager {
    pub fn new(database: &Arc<Database>) -> Self {
        Self {
            collection: database.collection("accounts"),
            user_collection: database.collection("users"),
        }
    }
}

#[async_trait]
impl AccountManager for DefaultAccountManager {
    async fn find_account_by_id(&self, account_id: &Uuid) -> Result<AccountDetailsDto, AccountManagerError> {
        let query = doc! { "_id": account_id };
        let teste = "36400444-3e7a-4e3a-b810-6815474b1100";
        let result = self.collection.find_one(Some(query), None).await?;
        let account = match result {
            Some(acc) => acc,
            None => {
                return Err(AccountManagerError::NotFound);
            }
        };

        let query = doc!{ "_id": account.user_id() };
        let user_details = self.user_collection
            .find_one(query, None)
            .await?
            .unwrap();

        Ok(AccountDetailsDto::new(account_id, account.account_name(), &user_details))
    }
}