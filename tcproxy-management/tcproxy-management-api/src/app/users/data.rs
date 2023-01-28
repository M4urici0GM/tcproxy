use mongodb::bson::{doc, Document, Uuid};
use mongodb::{Collection, Database};
use async_trait::async_trait;

use crate::app::bson::IntoUuid;
use crate::app::data::{RepositoryError, RepositoryReader, RepositoryResult, RepositoryWriter};
use crate::app::users::model::User;

#[async_trait]
pub trait UserRepositoryReader: RepositoryReader<User> {
    async fn exist_by_username_or_email(&self, username: &str, email: &str) -> RepositoryResult<bool>;
    async fn find_by_username(&self, username: &str) -> RepositoryResult<User>;
}

#[derive(Clone)]
pub struct UserRepositoryWriterImpl {
    collection: Collection<User>
}

#[derive(Clone)]
pub struct UserRepositoryReaderImpl {
    collection: Collection<User>
}

impl UserRepositoryWriterImpl {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection("users")
        }
    }
}

impl UserRepositoryReaderImpl {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection("users")
        }
    }
}

#[async_trait]
impl RepositoryWriter<User> for UserRepositoryWriterImpl {
    async fn insert_one(&self, entity: &User) -> RepositoryResult<Uuid> {
        let created_entity = self.collection
            .insert_one(entity, None)
            .await?;

        match created_entity.inserted_id.into_uuid() {
            Ok(id) => Ok(id),
            Err(err) => Err(RepositoryError::Other(err)),
        }
    }
}

#[async_trait]
impl RepositoryReader<User> for UserRepositoryReaderImpl {
    async fn find_by_id(&self, entity_id: Uuid) -> RepositoryResult<User> {
        let query = doc! { "_id": entity_id };
        let user = self.collection.find_one(Some(query), None).await?;

        match user {
            Some(user) => Ok(user),
            None => Err(RepositoryError::NotFound {
                message: format!("User {} not found", entity_id)
            })
        }
    }

    async fn find_one(&self, query: Document) -> RepositoryResult<User> {
        let user = self.collection.find_one(Some(query), None).await?;

        match user {
            Some(user) => Ok(user),
            None => Err(RepositoryError::NotFound { message: String::from("User not found") })
        }
    }
}

#[async_trait]
impl UserRepositoryReader for UserRepositoryReaderImpl {
    async fn exist_by_username_or_email(&self, username: &str, email: &str) -> RepositoryResult<bool> {
        let query = doc! {
            "$or": [
                { "username": username },
                { "email": email }
            ]
        };

        let result = self.collection
            .find_one(query, None)
            .await?;

        Ok(result.is_some())
    }

    async fn find_by_username(&self, username: &str) -> RepositoryResult<User> {
        let user = self.find_one(doc!{ "username": username }).await?;
        Ok(user)
    }
}

