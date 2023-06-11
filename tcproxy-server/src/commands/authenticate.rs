use std::{str::FromStr, sync::Arc};
use async_trait::async_trait;
use bcrypt::verify;
use chrono::{Utc, Duration};
use tcproxy_core::auth::User;
use uuid::Uuid;
use tokio::sync::mpsc::Sender;
use tracing::{info, error, debug};

use tcproxy_core::{AsyncCommand, Result, TcpFrame};
use tcproxy_core::auth::token_handler::{TokenHandler, Claims, AuthToken, TokenHandlerError};
use tcproxy_core::framing::{ AuthenticateAck, Error, Reason, GrantType, Authenticate, TokenAuthenticationArgs, PasswordAuthArgs};
use crate::managers::{UserManager, AccountManagerError, AuthenticationManagerGuard};


#[derive(Debug)]
pub struct AuthenticateCommandArgs {
    grant_type: GrantType,
}

impl From<Authenticate> for AuthenticateCommandArgs {
    fn from(value: Authenticate) -> Self {
        Self {
            grant_type: value.grant_type().to_owned()
        }
    }
}

pub enum AuthenticateCommandError {
    AuthenticationFailed,
    Other(tcproxy_core::Error)
}


impl From<TokenHandlerError> for AuthenticateCommandError {
    fn from(value: TokenHandlerError) -> Self {
        match value {
            TokenHandlerError::InvalidToken => Self::AuthenticationFailed,
            TokenHandlerError::Other(err) => Self::Other(err.into()),
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


/// Represents the event when client first connects to the server.
/// (Client) sends ClientConnected   ---> [Server]
/// (Server)                         <--- [Server] sends back ClientConnectedAck
/// (Client) sends Authenticate      ---> [Server]
/// (Client)                         <--- [Server] sends back AuthenticateAck
pub struct AuthenticateCommand {
    args: AuthenticateCommandArgs,
    client_sender: Sender<TcpFrame>,
    auth_manager: Arc<AuthenticationManagerGuard>,
    account_manager: Arc<Box<dyn UserManager + 'static>>,
    token_handler: Arc<Box<dyn TokenHandler + 'static>>,
}

impl AuthenticateCommand {
    pub fn new(
        args: AuthenticateCommandArgs,
        client_sender: &Sender<TcpFrame>,
        auth_manager: &Arc<AuthenticationManagerGuard>,
        token_handler: &Arc<Box<dyn TokenHandler + 'static>>,
        account_manager: &Arc<Box<dyn UserManager + 'static>>) -> Self

    {
        Self {
            args,
            auth_manager: auth_manager.clone(),
            client_sender: client_sender.clone(),
            token_handler: token_handler.clone(),
            account_manager: account_manager.clone(),
        }
    }

    pub fn boxed_new(
        args: AuthenticateCommandArgs,
        client_sender: &Sender<TcpFrame>,
        auth_manager: &Arc<AuthenticationManagerGuard>,
        token_handler: &Arc<Box<dyn TokenHandler + 'static>>,
        account_manager: &Arc<Box<dyn UserManager + 'static>>) -> Box<Self>
    {
        Box::new(Self::new(
            args,
            client_sender,
            auth_manager,
            token_handler,
            account_manager))
    }

    async fn authenticate(&self) -> std::result::Result<(User, Option<AuthToken>), AuthenticateCommandError> {
       match &self.args.grant_type {
            GrantType::PASSWORD(data) => {
                let user_details = authenticate_with_password(&data, &self.account_manager).await?;
                let token = create_user_token(&user_details, &self.token_handler)?;

                Ok((user_details, Some(token)))
            },
            GrantType::TOKEN(data) => {
                let user_details = authenticate_with_token(&data, &self.account_manager, &self.token_handler).await?;

                Ok((user_details, None))
            },
        }
    }
    
}

#[async_trait]
impl AsyncCommand for AuthenticateCommand {
    type Output = Result<()>;

    async fn handle(&mut self) -> Self::Output {
        if self.auth_manager.is_authenticated() {
            send_authentication_failed_frame(&self.client_sender, &Reason::AuthenticationFailed).await?;
            return Ok(());
        }

        let (user, token) = match self.authenticate().await {
            Ok(acc_details) => acc_details,
            Err(AuthenticateCommandError::AuthenticationFailed) => {
                send_authentication_failed_frame(&self.client_sender, &Reason::AuthenticationFailed).await?;
                return Ok(())
            }
            Err(AuthenticateCommandError::Other(err)) => {
                error!("failed when trying to fetch account details: {}", err);
                // At this point, we couldn't handle the error.
                // Send a error response to client, and close the connection.
                self.client_sender.send(TcpFrame::Error(Error::new(&Reason::UnexpectedError, &vec![]))).await?;
                return Err(format!("unexpected error: {:?}", err).into());
            }
        };

       
        info!("successfully authenticated, sending AuthenticateAck frame back");
        let authenticate_ack = AuthenticateAck::new(
            user.id().to_string().as_ref(),
            user.email(),
            token
        );

        self.auth_manager.set_authentication_details(&user);
        self.client_sender.send(TcpFrame::AuthenticateAck(authenticate_ack)).await?;

        Ok(())
    }
}

fn create_user_token(
    user: &User,
    token_handler: &Arc<Box<dyn TokenHandler + 'static>>) -> std::result::Result<AuthToken, AuthenticateCommandError>
{
    let now = Utc::now();
    let expiration = (now + Duration::hours(2)).timestamp_millis() as usize;
    let now = now.timestamp_millis() as usize;

    let claims = Claims::new(
        &expiration,
        &now,
        user.id().to_string().as_str(),
        "http://127.0.0.1:8080/",
        "http://127.0.0.1:8080/");


    Ok(token_handler.encode(&claims)?)
}

async fn authenticate_with_password(
    args: &PasswordAuthArgs,
    manager: &Arc<Box<dyn UserManager + 'static>>) -> std::result::Result<User, AuthenticateCommandError>
{
    let account_details = manager.find_user_by_email(args.username())?;
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
    manager: &Arc<Box<dyn UserManager + 'static>>,
    token_handler: &Arc<Box<dyn TokenHandler + 'static>>) -> std::result::Result<User,AuthenticateCommandError>
{
    let token = args.token();
    let maybe_claims = match token_handler.decode(token) {
        Ok(claims) => Some(claims),
        Err(_) => None
    };

    let token_details = match maybe_claims {
        Some(claims) => claims,
        None => {
            return Err(AuthenticateCommandError::AuthenticationFailed);
        }
    };

    let account_id = Uuid::from_str(token_details.sub()).unwrap(); // TODO: fix me

    info!("trying to find user with id: {}", account_id);
    let user_details = manager.find_account_by_id(&account_id)?;

    info!("successfully found user with id {}", account_id);
    Ok(user_details)
}


fn create_authentication_failed_frame(reason: &Reason) -> TcpFrame {
    TcpFrame::Error(Error::new(reason, &vec![]))
}

async fn send_authentication_failed_frame(sender: &Sender<TcpFrame>, reason: &Reason) -> Result<()> {
    let frame = create_authentication_failed_frame(reason);
    sender
        .send(frame)
        .await?;

    Ok(())
}
