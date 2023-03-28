use async_trait::async_trait;
use std::io::{stdout, Write};
use tracing::debug;

use tcproxy_core::framing::{PasswordAuthArgs, Authenticate, GrantType, Reason};
use tcproxy_core::transport::TcpFrameTransport;
use tcproxy_core::{Result, TcpFrame};
use tcproxy_core::AsyncCommand;

use crate::config::{AppConfig, AppContext};
use crate::server_addr::ServerAddr;
use crate::{LoginArgs, DefaultDirectoryResolver};

use super::contexts::DirectoryResolver;

pub struct LoginCommand {
    args: LoginArgs,
    app_cfg: AppConfig,                     //   |-> reason why abstract this into a manager
    dir_resolver: DefaultDirectoryResolver  // <-|
}

impl LoginCommand {
    pub fn new(args: &LoginArgs, cfg: &AppConfig, dir_resolver: &DefaultDirectoryResolver) -> Self {
        Self {
            args: args.clone(),
            app_cfg: cfg.clone(),
            dir_resolver: dir_resolver.clone(),
        }
    }

    fn get_context(&self) -> Result<AppContext> {
        let context_name = self
            .args
            .app_context()
            .clone()
            .unwrap_or(self.app_cfg.default_context_str().to_string());

        match self.app_cfg.get_context(&context_name) {
            Some(ctx) => Ok(ctx),
            None => Err(format!("context {} was not found.", context_name).into()),
        }
    }
}

#[async_trait]
impl AsyncCommand for LoginCommand {
    type Output = Result<()>;

    async fn handle(&mut self) -> Self::Output {
        // TODO: abstract the config into sort of a config manager?
        let config_path = self.dir_resolver.get_config_file()?;
        let mut config = AppConfig::load(&config_path)?;

        // gather all required information
        let password_args = get_password_auth_args(&self.args)?;
        let grant_type = GrantType::from(password_args);
        let authenticate_frame = TcpFrame::from(Authenticate::new(grant_type));

        // creates transport
        let app_context = self.get_context()?;
        let addr = ServerAddr::try_from(app_context)?.to_socket_addr()?;
        let mut transport = TcpFrameTransport::connect(addr).await?;

        match transport.send_frame(&authenticate_frame).await? {
            TcpFrame::AuthenticateAck(data) => {
                debug!("authenticated successfully");
                debug!("trying to save user token into config file..");

                // Stores user token into local config file
                config.set_user_token(data.token());
                AppConfig::save_to_file(&config, &config_path)?;

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
