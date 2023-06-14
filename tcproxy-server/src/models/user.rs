use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(
    Default, Clone, Debug, PartialEq, Serialize, Deserialize, Queryable, Selectable, Insertable,
)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct UserModel {
    #[serde(rename = "_id")]
    id: Vec<u8>,
    name: String,
    email: String,
    password_hash: String,
}

impl UserModel {
    pub fn new(id: &Uuid, name: &str, email: &str, password: &str) -> Self {
        Self {
            id: id.into_bytes().to_vec(),
            name: String::from(name),
            email: String::from(email),
            password_hash: String::from(password),
        }
    }

    pub fn id(&self) -> &[u8] {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn email(&self) -> &str {
        &self.email
    }

    pub fn password(&self) -> &str {
        &self.password_hash
    }
}
