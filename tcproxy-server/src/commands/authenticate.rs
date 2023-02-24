use std::sync::Arc;
use async_trait::async_trait;
use mongodb::bson::Uuid;
use tokio::sync::mpsc::Sender;
use tracing::{debug, error};

use tcproxy_core::{AsyncCommand, Result, TcpFrame};
use tcproxy_core::auth::token_handler::TokenHandler;
use tcproxy_core::framing::{Authenticate, AuthenticateAck, Error, Reason};
use crate::managers::{AccountManager, AccountManagerError, AuthenticationManagerGuard, DefaultAccountManager};

#[derive(Debug)]
pub struct AuthenticateArgs {
    account_token: String,
}

impl From<Authenticate> for AuthenticateArgs {
    fn from(value: Authenticate) -> Self {
        Self {
            account_token: String::from(value.token()),
        }
    }
}

pub struct AuthenticateCommand {
    args: AuthenticateArgs,
    client_sender: Sender<TcpFrame>,
    auth_manager: Arc<AuthenticationManagerGuard>,
    account_manager: Arc<Box<dyn AccountManager + 'static>>,
    token_handler: Arc<Box<dyn TokenHandler + 'static>>,
}

impl AuthenticateCommand {
    pub fn new(
        args: AuthenticateArgs,
        client_sender: &Sender<TcpFrame>,
        auth_manager: &Arc<AuthenticationManagerGuard>,
        token_handler: &Arc<Box<dyn TokenHandler + 'static>>,
        account_manager: &Arc<Box<dyn AccountManager + 'static>>) -> Self

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
        args: AuthenticateArgs,
        client_sender: &Sender<TcpFrame>,
        auth_manager: &Arc<AuthenticationManagerGuard>,
        token_handler: &Arc<Box<dyn TokenHandler + 'static>>,
        account_manager: &Arc<Box<dyn AccountManager + 'static>>) -> Box<Self>
    {
        Box::new(Self::new(
            args,
            client_sender,
            auth_manager,
            token_handler,
            account_manager))
    }
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

#[async_trait]
impl AsyncCommand for AuthenticateCommand {
    type Output = Result<()>;

    async fn handle(&mut self) -> Self::Output {
        if self.auth_manager.is_authenticated() {
            send_authentication_failed_frame(&self.client_sender, &Reason::AuthenticationFailed).await?;
            return Ok(());
        }

        let maybe_claims = match self.token_handler.decode(&self.args.account_token) {
            Ok(claims) => Some(claims),
            Err(_) => None
        };

        let token_details = match maybe_claims {
            Some(claims) => claims,
            None => {
                send_authentication_failed_frame(&self.client_sender, &Reason::AuthenticationFailed).await?;
                return Err("authentication failed".into());
            }
        };

        let account_id = Uuid::parse_str(token_details.sub())?;
        let account_details = match self.account_manager
            .find_account_by_id(&account_id)
            .await
        {
            Ok(acc_details) => acc_details,
            Err(AccountManagerError::NotFound) => {
                // Account was not found.
                // At this point, the client should show an error message, and close the connection.
                debug!("no account {} found!", account_id);
                send_authentication_failed_frame(&self.client_sender, &Reason::AuthenticationFailed).await?;
                return Err("authentication failed".into());
            },
            Err(AccountManagerError::Other(err)) => {
                error!("failed when trying to fetch account details: {}", err);
                // At this point, we couldn't handle the error.
                // Send a error response to client, and close the connection.
                self.client_sender
                    .send(TcpFrame::Error(Error::new(&Reason::UnexpectedError, &vec![])))
                    .await?;

                return Err("authentication failed".into());
            }
        };

        let authenticate_ack = AuthenticateAck::new(
            account_details.account_name(),
            account_details.user().email());

        self.auth_manager.set_authentication_details(&account_details);
        self.client_sender.send(TcpFrame::AuthenticateAck(authenticate_ack)).await?;

        Ok(())
    }
}