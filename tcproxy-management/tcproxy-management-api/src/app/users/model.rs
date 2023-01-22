use derive_builder::Builder;
use mongodb::bson::Uuid;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Builder, Serialize, Deserialize)]
#[builder(pattern = "immutable")]
pub struct User {
    #[serde(rename = "_id")]
    id: Uuid,
    name: String,
    username: String,
    email_address: String,
    password_hash: String,
}

impl User {
    pub fn new(
        id: Option<Uuid>,
        username: &str,
        name: &str,
        password_hash: &str,
        email_address: &str) -> Self {
        Self {
            id: id.unwrap_or(Uuid::new()),
            username: String::from(username),
            name: String::from(name),
            email_address: String::from(email_address),
            password_hash: String::from(password_hash),
        }
    }

    pub fn id(&self) -> &Uuid {
        &self.id
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn email(&self) -> &str {
        &self.email_address
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}