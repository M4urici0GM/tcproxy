use std::fmt::{Display, Formatter};
use actix_web::{http, HttpResponse, ResponseError};
use serde::{Deserialize, Serialize};

pub mod fmt;
pub mod app;

#[derive(Debug, Serialize, Deserialize)]
pub enum StatusCode {
    InternalServerError,
    NotFoundError,
    ConflictError,
    BadRequestError,
    Unauthorized,
    Forbidden,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationErrorDetails {
    property_name: String,
    message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HttpAppError {
    message: String,
    #[serde(skip_serializing)]
    error_type: StatusCode,
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

impl HttpAppError {
    pub fn new(
        message: &str,
        err_type: StatusCode,
        validation_err: Option<Vec<ValidationErrorDetails>>) -> Self {
        Self {
            message: String::from(message),
            error_type: err_type,
            validation_errors: validation_err,
        }
    }
}

impl Display for ValidationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let validation_errors = self.errors
            .to_vec()
            .iter()
            .fold(String::new(), |val, current| {
                format!("{}[property={}, message={}], ", val, current.property_name, current.message)
            });

        write!(f, "validation failed = {}, errors = {}", self.message, validation_errors)
    }
}

impl Display for HttpAppError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // same for this little bit.
        let msg = match self.error_type {
            StatusCode::NotFoundError => "not found",
            StatusCode::InternalServerError => "internal server error",
            StatusCode::ConflictError => "conflict",
            StatusCode::BadRequestError => "bad_request",
            StatusCode::Forbidden => "forbidden",
            StatusCode::Unauthorized => "unauthorized"
        };

        let validation_errors = match &self.validation_errors {
            Some(errors) => Vec::from(&errors[..]),
            None => Vec::new()
        };

        let validation_errors = validation_errors
            .to_vec()
            .iter()
            .fold(String::new(), |val, current| {
                format!("{}[property={}, message={}], ", val, current.property_name, current.message)
            });

        write!(
            f,
            "error_type = {}, message = {}, validation_errors = {}",
            msg,
            self.message,
            &validation_errors)
    }
}

impl ResponseError for HttpAppError {
    fn status_code(&self) -> http::StatusCode {
        // yeah, that's sounds a little bit redundant... maybe trying to use newtype here?
        match self.error_type {
            StatusCode::NotFoundError => http::StatusCode::NOT_FOUND ,
            StatusCode::BadRequestError => http::StatusCode::BAD_REQUEST,
            StatusCode::InternalServerError => http::StatusCode::INTERNAL_SERVER_ERROR,
            StatusCode::ConflictError => http::StatusCode::CONFLICT,
            StatusCode::Unauthorized => http::StatusCode::UNAUTHORIZED,
            StatusCode::Forbidden => http::StatusCode::FORBIDDEN,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let status_code = self.status_code();
        HttpResponse::build(status_code).json(self)
    }
}

impl From<tcproxy_core::Error> for HttpAppError {
    fn from(_: tcproxy_core::Error) -> Self {
        Self {
            message: String::from("Internal Server Error"),
            error_type: StatusCode::InternalServerError,
            validation_errors: None,
        }
    }
}

impl From<mongodb::error::Error> for HttpAppError {
    fn from(_: mongodb::error::Error) -> Self {
        Self {
            message: String::from("Internal Server Error"),
            error_type: StatusCode::InternalServerError,
            validation_errors: None,
        }
    }
}

impl From<ValidationError> for HttpAppError {
    fn from(value: ValidationError) -> Self {
        Self::new(
            value.message(),
            StatusCode::BadRequestError,
            Some(Vec::from(value.errors())),
        )
    }
}

impl std::error::Error for HttpAppError {
    
}

