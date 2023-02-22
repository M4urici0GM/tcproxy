use std::sync::Arc;
use async_trait::async_trait;
use tokio::sync::mpsc::Sender;

use tcproxy_core::{AsyncCommand, Result, TcpFrame};
use tcproxy_core::auth::token_handler::TokenHandler;
use tcproxy_core::framing::{Authenticate, AuthenticateAck, Error, Reason};
use crate::managers::{AccountManager, AccountManagerError, AuthenticationManagerGuard};

#[derive(Debug)]
pub struct AuthenticateArgs {
    account_id: String,
    account_token: String,
}

pub struct AuthenticateCommand {
    args: AuthenticateArgs,
    client_sender: Sender<TcpFrame>,
    auth_manager: Arc<AuthenticationManagerGuard>,
    account_manager: Arc<Box<dyn AccountManager + 'static>>,
    token_handler: Arc<Box<dyn TokenHandler + 'static>>,
}

impl From<Authenticate> for AuthenticateArgs {
    fn from(value: Authenticate) -> Self {
        Self {
            account_id: value.account_id().to_owned(),
            account_token: value.token().to_owned(),
        }
    }
}

impl AuthenticateCommand {
    pub fn new(
        args: AuthenticateArgs,
        client_sender: &Sender<TcpFrame>,
        auth_manager: &Arc<AuthenticationManagerGuard>,
        token_handler: &Arc<Box<dyn TokenHandler>>,
        account_manager: &Arc<Box<dyn AccountManager>>) -> Self

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
        token_handler: &Arc<Box<dyn TokenHandler>>,
        account_manager: &Arc<Box<dyn AccountManager>>) -> Box<Self>

    {
        Box::new(Self::new(
            args,
            client_sender,
            auth_manager,
            token_handler,
            account_manager))
    }
}

fn create_authentication_failed_frame() -> TcpFrame {
    TcpFrame::Error(Error::new(&Reason::AuthenticationFailed, &vec![]))
}

async fn send_authentication_failed_frame(sender: &Sender<TcpFrame>) -> Result<()> {
    let frame = create_authentication_failed_frame();
    sender
        .send(frame)
        .await?;

    Ok(())
}

#[async_trait]
impl AsyncCommand for AuthenticateCommand {
    type Output = Result<()>;

    async fn handle(&mut self) -> Self::Output {
        let token_details = match self.token_handler.decode(&self.args.account_token) {
            Ok(claims) => claims,
            Err(_) => {
                // sends back to client an error framing indicating that
                // authentication failed. at this point client should
                // close the connection and show an error to the user.
                send_authentication_failed_frame(&self.client_sender).await?;
                return Err("authentication failed".into());
            }
        };

        if self.args.account_id != token_details.sub() {
            send_authentication_failed_frame(&self.client_sender).await?;
            return Err("authentication failed".into());
        }

        let account_details = match self.account_manager
            .find_account_by_account_id(&self.args.account_id)
            .await
        {
            Ok(acc_details) => acc_details,
            Err(AccountManagerError::UserNotFound) => {
                // Account was not found.
                // At this point, the client should show an error message, and close the connection.
                send_authentication_failed_frame(&self.client_sender).await?;
                return Err("authentication failed".into());
            },
            Err(_) => {

                // At this point, we couldn't handle the error.
                // Send a error response to client, and close the connection.
                self.client_sender
                    .send(TcpFrame::Error(Error::new(&Reason::UnexpectedError, &vec![])))
                    .await?;

                return Err("authentication failed".into());
            }
        };

        let authenticate_ack = AuthenticateAck::new(
            account_details.id(),
            account_details.user().email());

        self.auth_manager.set_authentication_details(&account_details);
        self.client_sender
            .send(TcpFrame::AuthenticateAck(authenticate_ack))
            .await?;

        Ok(())
    }
}