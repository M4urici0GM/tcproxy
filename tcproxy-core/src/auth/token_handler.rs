use std::io::ErrorKind;
use serde::{Deserialize, Serialize};

use super::User;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    exp: usize,
    sub: String, // account id
    iss: String,
    aud: String,
    iat: usize,
}

impl Claims {
    pub fn iat(&self) -> &usize {
        &self.iat
    }

    pub fn exp(&self) -> &usize {
        &self.exp
    }

    pub fn sub(&self) -> &str {
        &self.sub
    }

    pub fn iss(&self) -> &str {
        &self.iss
    }

    pub fn aud(&self) -> &str {
        &self.aud
    }
}

pub enum TokenHandlerError {
    InvalidToken,
    Other(ErrorKind),
}

pub struct AuthToken(String);

impl AuthToken {
    pub fn new(token: &str) -> Self {
        Self(String::from(token))
    }

    pub fn get(&self) -> &str {
        &self.0
    }
}

pub trait TokenHandler: Sync + Send {
    fn encode(&self, claims: &Claims) -> Result<AuthToken, TokenHandlerError>;
    fn decode(&self, token: &str) ->  Result<Claims, TokenHandlerError>;
}
