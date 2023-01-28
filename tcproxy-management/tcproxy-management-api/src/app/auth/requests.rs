
use serde::{Deserialize, Serialize};
use crate::{ValidationError, ValidationErrorDetails, Validator};
use crate::app::users::requests::UserDto;

#[derive(Debug, Serialize, Deserialize)]
pub enum GrantType {
    Password,
    RefreshToken
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthenticateRequest {
    #[serde(default)]
    username: String,
    #[serde(default)]
    password: String,
}


#[derive(Debug, Default, Serialize, Deserialize)]
pub struct JwtTokenDto {
    token: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct AuthenticateResponse {
    user_details: UserDto,
    token: JwtTokenDto
}


impl AuthenticateRequest {
    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn password(&self) -> &str {
        &self.password
    }
}

impl Validator for AuthenticateRequest {
    fn validate(&self) -> Result<(), ValidationError> {
        let mut validation_errors = Vec::new();
        if self.username == String::default() {
            validation_errors.push(ValidationErrorDetails::new("username", "Username required"));
        }

        if self.password == String::default() {
            validation_errors.push(ValidationErrorDetails::new("password", "Password required"));
        }

        if !validation_errors.is_empty() {
            return Err(ValidationError::new("Validation for AuthenticateRequest failed", &validation_errors));
        }

        Ok(())
    }
}