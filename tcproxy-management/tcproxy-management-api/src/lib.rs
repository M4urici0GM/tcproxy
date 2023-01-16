use std::fmt::{Display, Formatter};
use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};
use serde::{Deserialize, Serialize};
use crate::AppErrorType::InternalServerError;

pub mod fmt;
pub mod app;

#[derive(Debug, Serialize, Deserialize)]
pub enum AppErrorType {
    InternalServerError,
    NotFoundError,
    ConflictError,
    BadRequestError,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationErrorDetails {
    property_name: String,
    message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct  AppError {
    message: String,
    #[serde(skip_serializing)]
    error_type: AppErrorType,
    #[serde(skip_serializing_if = "Option::is_none")]
    validation_errors: Option<Vec<ValidationErrorDetails>>
}

pub trait Validator {
    fn validate(&self) -> Result<(), ValidationError>;
}

#[derive(Debug, Default)]
pub struct ValidationError {
    message: String,
    errors: Vec<ValidationErrorDetails>
}

impl ValidationError {
    pub fn new(msg: &str, errors: &[ValidationErrorDetails]) -> Self {
        Self {
            message: String::from(msg),
            errors: errors.to_vec(),
        }
    }

    pub fn message(&self) -> &str { &self.message }

    pub fn errors(&self) -> &[ValidationErrorDetails] { &self.errors }
}

impl ValidationErrorDetails {
    pub fn new(property: &str, msg: &str) -> Self {
        Self {
            property_name: String::from(property),
            message: String::from(msg),
        }
    }
}

impl AppError {
    pub fn new(
        message: &str,
        err_type: AppErrorType,
        validation_err: Option<Vec<ValidationErrorDetails>>) -> Self {
        Self {
            message: String::from(message),
            error_type: err_type,
            validation_errors: validation_err,
        }
    }
}

impl Display for AppError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let msg = match self.error_type {
            AppErrorType::NotFoundError => "not found",
            AppErrorType::InternalServerError => "internal server error",
            AppErrorType::ConflictError => "conflict",
            AppErrorType::BadRequestError => "bad_request",
        };

        let validation_errors = match &self.validation_errors {
            Some(errors) => Vec::from(&errors[..]),
            None => Vec::new()
        };

        let validation_errors = validation_errors
            .to_vec()
            .iter()
            .fold(String::new(), |val, current| {
                format!("{}, [property={}, message={}]", val, current.property_name, current.message)
            });

        write!(
            f,
            "error_type = {}, message = {}, validation_errors = {}",
            msg,
            self.message,
            &validation_errors)
    }
}

impl ResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        match &self.error_type {
            AppErrorType::ConflictError => StatusCode::CONFLICT,
            AppErrorType::NotFoundError => StatusCode::NOT_FOUND,
            AppErrorType::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR,
            AppErrorType::BadRequestError => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let status_code = self.status_code();
        HttpResponse::build(status_code).json(self)
    }
}

impl From<tcproxy_core::Error> for AppError {
    fn from(_: tcproxy_core::Error) -> Self {
        Self {
            message: String::from("Internal Server Error"),
            error_type: InternalServerError,
            validation_errors: None,
        }
    }
}

impl From<mongodb::error::Error> for AppError {
    fn from(_: mongodb::error::Error) -> Self {
        Self {
            message: String::from("Internal Server Error"),
            error_type: InternalServerError,
            validation_errors: None,
        }
    }
}

impl From<ValidationError> for AppError {
    fn from(value: ValidationError) -> Self {
        Self::new(
            value.message(),
            AppErrorType::BadRequestError,
            Some(Vec::from(value.errors())),
        )
    }
}

