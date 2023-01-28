use async_trait::async_trait;
use bcrypt::verify;
use crate::app::auth::requests::{AuthenticateRequest, AuthenticateResponse};
use crate::app::core::command::{Command, CommandHandler};
use crate::app::data::error::AppError;
use crate::app::data::RepositoryError;
use crate::app::users::data::UserRepositoryReader;
use crate::Validator;

pub struct AuthenticateRequestHandler {
    user_reader: Box<dyn UserRepositoryReader>
}

impl AuthenticateRequestHandler {
    pub fn new<T: UserRepositoryReader + 'static>(reader: T) -> Self {
        Self {
            user_reader: Box::new(reader),
        }
    }
}

impl Command for AuthenticateRequest {
    type Output = Result<AuthenticateResponse, AppError>;
}

#[async_trait]
impl CommandHandler<AuthenticateRequest> for AuthenticateRequestHandler {
    async fn execute_cmd(&self, cmd: AuthenticateRequest) -> Result<AuthenticateResponse, AppError> {
        cmd.validate()?;
        // i think that's why we should use Enum errors and impl the From<T> trait
        // is way better impl the trait, than manually map the error like this.
        // and yeah... i didnt like this next piece.
        // TODO: trying to create default implementations for each internal type.
        let user = match self.user_reader
            .find_by_username(cmd.username())
            .await
        {
            Ok(user ) => user,
            Err(RepositoryError::NotFound { .. }) => {
                return Err(AppError::InvalidCredentials {
                    message: Some(String::from("Invalid username or password.")),
                })
            },
            Err(err) => {
                return Err(AppError::from(err));
            }
        };

        if !verify(cmd.password(), user.password_hash())? {
            return Err(AppError::InvalidCredentials {
                message: Some(String::from("Invalid username or password."))
            });
        }

        // TODO: create user's token.

        Ok(AuthenticateResponse::default())
    }
}

// TODO: write some unit tests.