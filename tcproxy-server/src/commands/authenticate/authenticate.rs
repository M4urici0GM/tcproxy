use bcrypt::verify;
use chrono::{Duration, Utc};
use std::{str::FromStr, sync::Arc};
use tcproxy_core::auth::User;
use tokio::sync::mpsc::Sender;
use tracing::{error, info};
use uuid::Uuid;

use crate::managers::AccountManagerError;
use crate::proxy::DefaultTokenHandler;
use crate::ClientState;
use tcproxy_core::auth::token_handler::{AuthToken, Claims, TokenHandler, TokenHandlerError};
use tcproxy_core::framing::{
    Authenticate, AuthenticateAck, Error, GrantType, PasswordAuthArgs, Reason,
    TokenAuthenticationArgs,
};
use tcproxy_core::{Result, TcpFrame};

#[derive(Debug)]
pub struct AuthenticateCommandArgs {
    grant_type: GrantType,
}

impl From<Authenticate> for AuthenticateCommandArgs {
    fn from(value: Authenticate) -> Self {
        Self {
            grant_type: value.grant_type().to_owned(),
        }
    }
}

pub enum AuthenticateCommandError {
    AuthenticationFailed,
    Other(tcproxy_core::Error),
}

impl From<TokenHandlerError> for AuthenticateCommandError {
    fn from(value: TokenHandlerError) -> Self {
        match value {
            TokenHandlerError::InvalidToken => Self::AuthenticationFailed,
            TokenHandlerError::Other(err) => Self::Other(err),
        }
    }
}

impl From<AccountManagerError> for AuthenticateCommandError {
    fn from(value: AccountManagerError) -> Self {
        match value {
            AccountManagerError::NotFound => Self::AuthenticationFailed,
            AccountManagerError::Other(err) => Self::Other(err),
        }
    }
}

impl From<bcrypt::BcryptError> for AuthenticateCommandError {
    fn from(value: bcrypt::BcryptError) -> Self {
        Self::Other(value.into())
    }
}

pub async fn challenge(
    grant_type: &GrantType,
    state: &Arc<ClientState>,
) -> std::result::Result<(User, Option<AuthToken>), AuthenticateCommandError> {
    match grant_type {
        GrantType::PASSWORD(data) => {
            let user_details = authenticate_with_password(data, state).await?;
            let token = create_user_token(&user_details, state)?;

            Ok((user_details, Some(token)))
        }
        GrantType::TOKEN(data) => {
            let user_details = authenticate_with_token(data, state).await?;

            Ok((user_details, None))
        }
    }
}

fn create_user_token(
    user: &User,
    state: &Arc<ClientState>,
) -> std::result::Result<AuthToken, AuthenticateCommandError> {
    let token_handler = DefaultTokenHandler::new(state.get_server_config());
    let now = Utc::now();
    let expiration = (now + Duration::hours(2)).timestamp_millis() as usize;
    let now = now.timestamp_millis() as usize;

    let claims = Claims::new(
        &expiration,
        &now,
        user.id().to_string().as_str(),
        "http://127.0.0.1:8080/",
        "http://127.0.0.1:8080/",
    );

    Ok(token_handler.encode(&claims)?)
}

async fn authenticate_with_password(
    args: &PasswordAuthArgs,
    state: &Arc<ClientState>,
) -> std::result::Result<User, AuthenticateCommandError> {
    let account_manager = state.get_accounts_manager();
    let account_details = account_manager.find_user_by_email(args.username())?;
    let user_hash = account_details.password();

    if !verify(args.password(), user_hash)? {
        // Invalid password.
        // At this point the client must show the error and ask again for credentials.
        return Err(AuthenticateCommandError::AuthenticationFailed);
    }

    Ok(account_details)
}

async fn authenticate_with_token(
    args: &TokenAuthenticationArgs,
    state: &Arc<ClientState>,
) -> std::result::Result<User, AuthenticateCommandError> {
    let token_handler = DefaultTokenHandler::new(state.get_server_config());
    let account_manager = state.get_accounts_manager();

    let maybe_claims = match token_handler.decode(args.token()) {
        Ok(claims) => Some(claims),
        Err(_) => None,
    };

    let token_details = match maybe_claims {
        Some(claims) => claims,
        None => {
            return Err(AuthenticateCommandError::AuthenticationFailed);
        }
    };

    let account_id = Uuid::from_str(token_details.sub()).unwrap(); // TODO: fix me

    info!("trying to find user with id: {}", account_id);
    let user_details = account_manager.find_account_by_id(&account_id)?;

    info!("successfully found user with id {}", account_id);
    Ok(user_details)
}
