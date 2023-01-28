use async_trait::async_trait;
use bcrypt::{DEFAULT_COST, hash};

use crate::app::core::command::{Command, CommandHandler};
use crate::app::data::error::AppError;
use crate::app::data::RepositoryWriter;
use crate::app::users::data::UserRepositoryReader;
use crate::app::users::model::User;
use crate::app::users::requests::CreateUserRequest;
use crate::Validator;

pub struct CreateUserCommandHandler {
    writer:  Box<dyn RepositoryWriter<User>>,
    reader: Box<dyn UserRepositoryReader>,
}

impl Command for CreateUserRequest {
    type Output = Result<User, AppError>;
}

impl CreateUserCommandHandler {
    pub fn new<Reader, Writer>(reader: &Reader, writer: &Writer) -> Self
        where
            Reader: UserRepositoryReader + Clone + 'static,
            Writer: RepositoryWriter<User> + Clone + 'static
    {
        Self {
            reader: Box::new(reader.clone()),
            writer: Box::new(writer.clone()),
        }
    }
}

#[async_trait]
impl CommandHandler<CreateUserRequest> for CreateUserCommandHandler {
    async fn execute_cmd(&self, cmd: CreateUserRequest) -> Result<User, AppError> {
        cmd.validate()?;
        let user_exists = self.reader
            .exist_by_username_or_email(cmd.username(), cmd.email())
            .await?;

        if user_exists {
            return Err(AppError::EntityAlreadyExists {
                message: format!(
                    "User with email {} or username {} already exists.",
                    cmd.email(),
                    cmd.username())
            });
        }

        let password_hash = hash(cmd.password(), DEFAULT_COST)?;
        let user = User::new(
            None,
            cmd.username(),
            cmd.name(),
            &password_hash,
            cmd.email());

        self.writer.insert_one(&user).await?;

        Ok(user)
    }
}