use std::sync::Arc;
use jsonwebtoken::{decode, DecodingKey, Validation};
use tracing::warn;
use tcproxy_core::auth::token_handler::{Claims, TokenHandler, TokenHandlerError};
use crate::ServerConfig;

pub struct  DefaultTokenHandler {
    server_config: Arc<ServerConfig>
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

    pub fn get_secret(&self) -> &DecodingKey {
        let secret = self.server_config.get_jwt_secret();
        &DecodingKey::from_secret(secret.as_bytes())
    }
}

impl TokenHandler for DefaultTokenHandler {
    fn decode(&mut self, token: &str) -> Result<Claims, TokenHandlerError> {
        let secret = self.get_secret();

        // TODO: implement custom validation for JWT token.
        match decode::<Claims>(token, secret, &Validation::default()) {
            Ok(data) => Ok(data.claims),
            Err(err) => {
                warn!("error trying to decode the token: {}", err);
                Err(TokenHandlerError::InvalidToken)
            },
        }
    }
}