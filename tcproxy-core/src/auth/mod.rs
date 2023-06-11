pub mod token_handler;

use uuid::Uuid;
use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct User {
    #[serde(rename = "_id")]
    id: Uuid,
    name: String,
    email: String,
    #[serde(rename = "passwordHash")]
    password: String,
}

impl User {
    pub fn new(id: &Uuid, name: &str, email: &str, password: &str) -> Self {
        Self {
            id: *id,
            name: String::from(name),
            email: String::from(email),
            password: String::from(password),
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

    pub fn password(&self) -> &str {
        &self.password
    }
}
