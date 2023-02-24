pub mod token_handler;

use mongodb::bson::Uuid;
use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UserDetails {
    #[serde(rename = "_id")]
    id: Uuid,
    name: String,
    email: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    #[serde(rename = "_id")]
    id: Uuid,
    account_name: String,
    user_id: Uuid,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AccountDetailsDto {
    #[serde(rename = "_id")]
    id: Uuid,
    #[serde(rename = "accountName")]
    account_name: String,
    #[serde(rename = "userId")]
    user_id: Uuid,
    #[serde(skip_serializing)]
    user_details: UserDetails
}

impl UserDetails {
    pub fn new(id: &Uuid, name: &str, email: &str) -> Self {
        Self {
            id: *id,
            name: String::from(name),
            email: String::from(email),
        }
    }

    pub fn id(&self) -> &Uuid {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn email(&self) -> &str {
        &self.email
    }
}

impl Account {
    pub fn id(&self) -> &Uuid {
        &self.id
    }

    pub fn user_id(&self) -> &Uuid {
        &self.user_id
    }

    pub fn account_name(&self) -> &str {
        &self.account_name
    }
}

impl AccountDetailsDto {
    pub fn new(id: &Uuid, name: &str, user_details: &UserDetails) -> Self {
        Self {
            id: *id,
            account_name: String::from(name),
            user_id: *user_details.id(),
            user_details: user_details.clone()
        }
    }

    pub fn id(&self) -> &Uuid {
        &self.id
    }

    pub fn user_id(&self) -> &Uuid {
        &self.user_id
    }

    pub fn account_name(&self) -> &str {
        &self.account_name
    }

    pub fn user(&self) -> &UserDetails {
        &self.user_details
    }
}