use lazy_static::lazy_static;
use fancy_regex::Regex;
use mongodb::bson::Uuid;
use serde::{Deserialize, Serialize};
use crate::{ValidationError, ValidationErrorDetails, Validator};
use crate::app::users::model::User;

lazy_static! {
    static ref USERNAME_REGEX: Regex = Regex::new(r"^(?!.*\.\.)(?!.*\.$)[^\W][\w.]{0,29}$").unwrap();
    static ref PASSWORD_REGEX: Regex = Regex::new(r"^(?=.*[A-Za-z])(?=.*\d)(?=.*[@$!%*#?&])[A-Za-z\d@$!%*#?&]{6,30}$").unwrap();
    static ref EMAIL_REGEX: Regex = Regex::new(r"[a-z0-9!#$%&'*+/=?^_`{|}~-]+(?:\.[a-z0-9!#$%&'*+/=?^_`{|}~-]+)*@(?:[a-z0-9](?:[a-z0-9-]*[a-z0-9])?\.)+[a-z0-9](?:[a-z0-9-]*[a-z0-9])?").unwrap();
}

const INVALID_USERNAME_MSG: &str = "Invalid username. Usernames must not contain special chars.";
const INVALID_PASSWORD_MSG: &str = "Password must have at least 6 and maximum of 30 characters, at least one number and one special character.";
const INVALID_EMAIL_MSG: &str = "Invalid email address.";
const INVALID_NAME_MSG: &str = "Invalid name, must have at least 3 chars.";

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CreateUserRequest {
    #[serde(default)]
    name: String,
    #[serde(default)]
    email: String,
    #[serde(default)]
    username: String,
    #[serde(default)]
    password: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UserDto {
    id: Uuid,
    name: String,
    username: String,
    email_address: String,
}

impl From<User> for UserDto {
    fn from(user: User) -> Self {
        Self {
            id: *user.id(),
            name: String::from(user.name()),
            username: String::from(user.username()),
            email_address: String::from(user.email()),
        }
    }
}

impl CreateUserRequest {
    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn email(&self) -> &str { &self.email }

    pub fn password(&self) -> &str { &self.password }
}

impl Validator for CreateUserRequest {
    fn validate(&self) -> Result<(), ValidationError> {
        let mut errors = Vec::new();
        if !USERNAME_REGEX.is_match(self.username()).unwrap() {
            errors.push(ValidationErrorDetails::new("username", INVALID_USERNAME_MSG));
        }

        if !PASSWORD_REGEX.is_match(self.password()).unwrap() {
            errors.push(ValidationErrorDetails::new("password", INVALID_PASSWORD_MSG));
        }

        if !EMAIL_REGEX.is_match(self.email()).unwrap() {
            errors.push(ValidationErrorDetails::new("email", INVALID_EMAIL_MSG));
        }

        if self.name().len() < 3 {
            errors.push(ValidationErrorDetails::new("name", INVALID_NAME_MSG));
        }

        if !errors.is_empty() {
            return Err(ValidationError::new("Create User validation failed.", &errors));
        }

        Ok(())
    }
}
