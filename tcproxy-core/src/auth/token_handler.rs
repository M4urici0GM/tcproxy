use std::io::ErrorKind;
use std::fmt::Display;
use serde::{Deserialize, Serialize};

use crate::Error;


#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    exp: usize,
    sub: String, // account id
    iss: String,
    aud: String,
    iat: usize,
}

impl Claims {
    pub fn new(
        exp: &usize,
        iat: &usize,
        sub: &str,
        iss: &str,
        aud: &str,
    ) -> Self {
        Self {
            exp: *exp,
            iat: *iat,
            sub: String::from(sub),
            iss: String::from(iss),
            aud: String::from(aud),
        }
    }

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

#[derive(Debug)]
pub enum TokenHandlerError {
    InvalidToken,
    Other(Error),
}

impl std::fmt::Display for TokenHandlerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl std::error::Error for TokenHandlerError {
    
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
