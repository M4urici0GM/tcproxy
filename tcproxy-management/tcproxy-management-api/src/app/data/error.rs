use std::error::Error;
use bcrypt::BcryptError;
use crate::{HttpAppError, StatusCode, ValidationError};
use crate::app::data::RepositoryError;

#[derive(Debug)]
pub enum AppError {
    Forbidden { message: String },
    InvalidCredentials { message: Option<String> },
    EntityNotFound { message: String },
    EntityAlreadyExists { message: String },
    EntityValidationError { error: ValidationError },
    Other { message: String, cause: Box<dyn Error> }
}

impl From<ValidationError> for AppError {
    fn from(value: ValidationError) -> Self {
        Self::EntityValidationError {
            error: value
        }
    }
}

impl From<AppError> for HttpAppError {
    fn from(value: AppError) -> Self {
        match value {
            AppError::EntityValidationError { error } => {
                HttpAppError::new(error.message(), StatusCode::BadRequestError, Some(Vec::from(error.errors())))
            },
            AppError::EntityAlreadyExists { message } => {
                HttpAppError::new(&message, StatusCode::ConflictError, None)
            },
            AppError::EntityNotFound { message } => {
                 HttpAppError::new(&message, StatusCode::NotFoundError, None)
            },
            AppError::Other { message, .. } => {
                let msg = format!("Internal server error. {}", message);
                HttpAppError::new(&msg, StatusCode::InternalServerError, None)
            },
            AppError::InvalidCredentials { message } => {
                let msg = message.unwrap_or_else(|| "Not Authorized.".to_string());
                HttpAppError::new(&msg, StatusCode::Unauthorized, None)
            },
            AppError::Forbidden { message } => {
                let msg = format!("Forbidden. {}", message);
                HttpAppError::new(&msg, StatusCode::Forbidden, None)
            }
        }
    }
}

impl From<RepositoryError> for AppError {
    fn from(value: RepositoryError) -> Self {
        match value {
            RepositoryError::NotFound { message } => Self::EntityNotFound { message },
            RepositoryError::Other(err) => Self::Other {
                message: format!("Error when trying to interact with database: {}", err),
                cause: err,
            }
        }
    }
}

impl<T: Error + 'static> From<T> for AppError {
    fn from(value: T) -> Self {
        AppError::Other { message: value.to_string(), cause: value.into() }
    }
}

impl From<BcryptError> for HttpAppError {
    fn from(value: BcryptError) -> Self {
        let msg = format!("Internal Server Error: {}", value);
        HttpAppError::new(&msg, StatusCode::InternalServerError, None)
    }
}