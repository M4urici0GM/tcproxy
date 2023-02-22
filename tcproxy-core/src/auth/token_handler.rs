use std::io::ErrorKind;
use mockall::automock;
use serde::{Deserialize, Serialize};

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


#[automock]
pub trait TokenHandler: Sync + Send {
    fn decode(&mut self, token: &str) ->  Result<Claims, TokenHandlerError>;
}
