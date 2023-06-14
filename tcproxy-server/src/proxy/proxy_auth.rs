use crate::ServerConfig;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use std::sync::Arc;
use tcproxy_core::auth::token_handler::{AuthToken, Claims, TokenHandler, TokenHandlerError};

use tracing::warn;

#[derive(Clone)]
pub struct DefaultTokenHandler {
    server_config: Arc<ServerConfig>,
}

impl DefaultTokenHandler {
    pub fn new(server_config: &Arc<ServerConfig>) -> Self {
        Self {
            server_config: server_config.clone(),
        }
    }

    pub fn boxed_new(server_config: &Arc<ServerConfig>) -> Box<Self> {
        let local_self = Self::new(server_config);
        Box::new(local_self)
    }

    pub fn get_decoding_secret(&self) -> DecodingKey {
        let secret = self.server_config.get_jwt_secret();
        DecodingKey::from_secret(secret.as_bytes())
    }

    pub fn get_encoding_secret(&self) -> EncodingKey {
        let secret = self.server_config.get_jwt_secret();
        EncodingKey::from_secret(secret.as_bytes())
    }
}

impl TokenHandler for DefaultTokenHandler {
    fn encode(&self, claims: &Claims) -> Result<AuthToken, TokenHandlerError> {
        let secret = self.get_encoding_secret();
        let header = Header::default();

        match encode(&header, claims, &secret) {
            Ok(token) => Ok(AuthToken::new(&token)),
            Err(err) => {
                warn!("error trying to encode the token: {}", err);
                Err(TokenHandlerError::InvalidToken)
            }
        }
    }

    fn decode(&self, token: &str) -> Result<Claims, TokenHandlerError> {
        let secret = self.get_decoding_secret();

        // TODO: implement custom validation for JWT token.
        match decode::<Claims>(token, &secret, &Validation::default()) {
            Ok(data) => Ok(data.claims),
            Err(err) => {
                warn!("error trying to decode the token: {}", err);
                Err(TokenHandlerError::InvalidToken)
            }
        }
    }
}
