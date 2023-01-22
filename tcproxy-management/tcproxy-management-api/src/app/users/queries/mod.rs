use async_trait::async_trait;
use mongodb::bson::Uuid;

use crate::app::core::command::{Command, CommandHandler};
use crate::app::data::RepositoryReader;
use crate::app::users::model::User;
use crate::AppErrorType;
use crate::app::data::error::EntityError;

#[derive(Debug)]
pub struct GetUserRequest {
    user_id: Uuid,
}

pub struct GetUserRequestHandler {
    reader: Box<dyn RepositoryReader<User>>
}

impl GetUserRequestHandler {
    pub fn new<Reader>(reader: &Reader) -> Self
    where
        Reader: RepositoryReader<User> + Clone + 'static
    {
        Self {
            reader: Box::new(reader.clone()),
        }
    }
}

impl GetUserRequest {
    pub fn new(user_id: &Uuid) -> Self {
        Self {
            user_id: *user_id,
        }
    }
}

impl Command for GetUserRequest {
    type Output = Result<User, EntityError>;
}

#[async_trait]
impl CommandHandler<GetUserRequest> for GetUserRequestHandler {
    async fn execute_cmd(&self, cmd: GetUserRequest) -> Result<User, EntityError> {
        let user = self.reader
            .find_by_id(cmd.user_id)
            .await?;

        Ok(user)
    }
}
