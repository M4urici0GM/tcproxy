use std::error::Error;
use bcrypt::BcryptError;
use crate::{AppError, AppErrorType, ValidationError};
use crate::app::data::RepositoryError;

#[derive(Debug)]
pub enum EntityError {
    EntityNotFound { message: String },
    EntityAlreadyExists { message: String },
    EntityValidationError { error: ValidationError },
    Other { message: String, cause: Box<dyn Error> }
}

impl From<ValidationError> for EntityError {
    fn from(value: ValidationError) -> Self {
        Self::EntityValidationError {
            error: value
        }
    }
}

impl From<EntityError> for AppError {
    fn from(value: EntityError) -> Self {
        match value {
            EntityError::EntityValidationError { error } => {
                AppError::new(error.message(), AppErrorType::BadRequestError, Some(Vec::from(error.errors())))
            },
            EntityError::EntityAlreadyExists { message } => {
                AppError::new(&message, AppErrorType::ConflictError, None)
            },
            EntityError::EntityNotFound { message } => {
                 AppError::new(&message, AppErrorType::NotFoundError, None)
            },
            EntityError::Other { message, .. } => {
                let msg = format!("Internal server error. {}", message);
                AppError::new(&msg, AppErrorType::InternalServerError, None)
            }
        }
    }
}

impl From<RepositoryError> for EntityError {
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

impl<T: Error + 'static> From<T> for EntityError {
    fn from(value: T) -> Self {
        EntityError::Other { message: value.to_string(), cause: value.into() }
    }
}

impl From<BcryptError> for AppError {
    fn from(value: BcryptError) -> Self {
        let msg = format!("Internal Server Error: {}", value);
        AppError::new(&msg, AppErrorType::InternalServerError, None)
    }
}