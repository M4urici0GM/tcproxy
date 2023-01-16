use futures_util::Stream;
use mongodb::bson::{doc, Uuid};
use mongodb::{Collection, Database};
use crate::app::bson::IntoUuid;

use crate::app::users::User;
use crate::{AppError, AppErrorType};

#[derive(Clone)]
pub struct UserRepository {
    collection: Collection<User>
}

impl UserRepository {
    pub fn new(mongodb: &Database) -> Self {
        Self {
            collection: mongodb.collection("users")
        }
    }

    pub async fn new_user(&self, user: User) -> Result<Uuid, AppError> {
        let created_user = self.collection
            .insert_one(&user, None)
            .await?;

        Ok(created_user.inserted_id.into_uuid()?)
    }

    pub async fn exist_by_username_or_email(&self, username: &str, email: &str) -> Result<bool, AppError> {
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

    pub async fn get_user(&self, user_id: Uuid) -> Result<User, AppError> {
        let query = doc! { "_id": user_id };
        let user = self.collection.find_one(Some(query), None).await?;

        match user {
            Some(user) => Ok(user),
            None => {
                let msg = format!("user {} not found", user_id);
                Err(AppError::new(&msg, AppErrorType::NotFoundError, None))
            }
        }
    }
}

