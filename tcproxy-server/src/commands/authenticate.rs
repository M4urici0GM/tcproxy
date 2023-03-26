use std::sync::Arc;
use async_trait::async_trait;
use bcrypt::verify;
use mongodb::bson::Uuid;
use tcproxy_core::auth::User;
use tokio::sync::mpsc::Sender;
use tracing::error;

use tcproxy_core::{AsyncCommand, Result, TcpFrame};
use tcproxy_core::auth::token_handler::{TokenHandler, Claims, AuthToken};
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

impl<T: std::error::Error + Send + Sync + 'static> From<T> for AuthenticateCommandError {
    fn from(value: T) -> Self {
        Self::Other(value.into())
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
}

#[async_trait]
impl AsyncCommand for AuthenticateCommand {
    type Output = Result<()>;

    async fn handle(&mut self) -> Self::Output {
        if self.auth_manager.is_authenticated() {
            send_authentication_failed_frame(&self.client_sender, &Reason::AuthenticationFailed).await?;
            return Ok(());
        }

        let auth_result = match &self.args.grant_type {
            GrantType::PASSWORD(data) => {
                authenticate_with_password(&data, &self.client_sender, &self.account_manager).await
            },
            GrantType::TOKEN(data) => {
                authenticate_with_token(&data, &self.account_manager, &self.client_sender, &self.token_handler).await
            },
        };

        let account_details = match auth_result {
            Ok(acc_details) => acc_details,
            Err(AuthenticateCommandError::AuthenticationFailed) => {
                send_authentication_failed_frame(&self.client_sender, &Reason::AuthenticationFailed).await?;
                return Err("authentication failed".into())
            }
            Err(AuthenticateCommandError::Other(err)) => {
                error!("failed when trying to fetch account details: {}", err);
                // At this point, we couldn't handle the error.
                // Send a error response to client, and close the connection.
                self.client_sender.send(TcpFrame::Error(Error::new(&Reason::UnexpectedError, &vec![]))).await?;
                return Err(format!("unexpected error: {:?}", err).into());
            }
        };

        let claims = Claims::from(user);
        let token = self.token_handler.encode()?;
        let authenticate_ack = AuthenticateAck::new(
            &account_details.id().to_string(),
            account_details.email(),
            token.get()
        );

        self.auth_manager.set_authentication_details(&account_details);
        self.client_sender.send(TcpFrame::AuthenticateAck(authenticate_ack)).await?;

        Ok(())
    }
}

fn create_user_token(user: &User, token_handler: &Arc<Box<dyn TokenHandler + 'static>>) -> Result<AuthToken> {
    
}

async fn authenticate_with_password(
    args: &PasswordAuthArgs,
    sender: &Sender<TcpFrame>,
    manager: &Arc<Box<dyn UserManager + 'static>>) -> std::result::Result<tcproxy_core::auth::User, AuthenticateCommandError>
{
    let account_details = manager.find_user_by_email(args.username()).await?;
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
    sender: &Sender<TcpFrame>,
    token_handler: &Arc<Box<dyn TokenHandler + 'static>>) -> std::result::Result<tcproxy_core::auth::User, AuthenticateCommandError>
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

    let account_id = Uuid::parse_str(token_details.sub())?;
    let user_details = manager.find_account_by_id(&account_id).await?;
    
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