use async_trait::async_trait;
use std::io::{stdout, Write};
use tcproxy_core::auth::token_handler::AuthToken;
use tracing::debug;

use tcproxy_core::framing::{Authenticate, GrantType, PasswordAuthArgs, Reason};
use tcproxy_core::transport::TcpFrameTransport;
use tcproxy_core::AsyncCommand;
use tcproxy_core::{Result, TcpFrame};

use crate::config::{AppContext, Config};
use crate::server_addr::ServerAddr;
use crate::LoginArgs;

pub struct LoginCommand {
    args: LoginArgs,
    config: Config,
}

impl LoginCommand {
    pub fn new(args: &LoginArgs, config: &Config) -> Self {
        Self {
            args: args.clone(),
            config: config.clone(),
        }
    }
}

#[async_trait]
impl AsyncCommand for LoginCommand {
    type Output = Result<()>;

    async fn handle(&mut self) -> Self::Output {
        // gather all required information
        let password_args = get_password_auth_args(&self.args)?;
        let grant_type = GrantType::from(password_args);
        let authenticate_frame = TcpFrame::from(Authenticate::new(grant_type));

        // creates transport
        let app_context = get_context(&self.args, &self.config).await?;
        let addr = ServerAddr::try_from(app_context)?.to_socket_addr()?;
        let mut transport = TcpFrameTransport::connect(addr).await?;

        match transport.send_frame(&authenticate_frame).await? {
            TcpFrame::AuthenticateAck(data) => {
                debug!("authenticated successfully");
                debug!("trying to save user token into config file..");

                // Stores user token into local config file
                let mut auth_manager = self.config.lock_auth_manager()?;
                auth_manager.set_current_token(Some(AuthToken::from(data.token())));

                Ok(())
            }
            TcpFrame::Error(err) if *err.reason() == Reason::AuthenticationFailed => {
                Err("Authentication failed. Try logging again with tcproxy-cli login".into())
            }
            actual => {
                debug!("received invalid frame when doing handshake. received {} instead of ClientConnectedAck", actual);
                Err("Error while trying to communicate with server.".into())
            }
        }
    }
}

async fn get_context(args: &LoginArgs, config: &Config) -> Result<AppContext> {
    let contexts = config.lock_context_manager()?;
    let context_name = args
        .app_context()
        .clone()
        .unwrap_or(contexts.default_context_str().to_string());

    match contexts.get_context(&context_name) {
        Some(ctx) => Ok(ctx),
        None => Err(format!("context {} was not found.", context_name).into()),
    }
}

fn get_password_auth_args(args: &LoginArgs) -> Result<PasswordAuthArgs> {
    let username = get_username(args)?;
    let password = rpassword::prompt_password("Your password: ")?;
    return Ok(PasswordAuthArgs::new(
        strip_newline(&username),
        &password,
        None,
    ));
}

fn get_username(args: &LoginArgs) -> Result<String> {
    match args.username() {
        Some(u) => Ok(u.to_owned()),
        None => {
            print!("Your email: ");
            stdout().flush()?;

            let mut username = String::default();
            let total_chars = std::io::stdin().read_line(&mut username)?;
            if 0 == total_chars {
                return Err("Invalid email.".into());
            }

            Ok(username)
        }
    }
}

fn strip_newline(input: &str) -> &str {
    input
        .strip_suffix("\r\n")
        .or(input.strip_suffix("\n"))
        .unwrap_or(input)
}
