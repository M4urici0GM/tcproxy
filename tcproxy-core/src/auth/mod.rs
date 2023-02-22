pub mod token_handler;

use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UserDetails {
    #[serde(rename = "_id")]
    id: String,
    name: String,
    email: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AccountDetails {
    #[serde(rename = "_id")]
    id: String,
    user: UserDetails,
}


impl UserDetails {
    pub fn new(id: &str, name: &str, email: &str) -> Self {
        Self {
            id: String::from(id),
            name: String::from(name),
            email: String::from(email),
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn email(&self) -> &str {
        &self.email
    }
}

impl AccountDetails {
    pub fn new(id: &str, user_details: &UserDetails) -> Self {
        Self {
            id: String::from(id),
            user: user_details.clone(),
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn user(&self) -> &UserDetails {
        &self.user
    }
}