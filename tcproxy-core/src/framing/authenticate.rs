use crate::framing::frame_types::AUTHENTICATE;
use crate::framing::utils::assert_connection_type;
use crate::io::get_u16;
use crate::{Frame, FrameDecodeError, PutU32String, ReadU32String};
use bytes::buf::BufMut;
use bytes::Buf;
use std::io::Cursor;

use super::authentication_grant_types::{AUTH_TOKEN_AUTHENTICATION, PASSWORD_AUTHENTICATION};

#[derive(Debug, PartialEq, Clone)]
pub enum GrantType {
    PASSWORD(PasswordAuthArgs),
    TOKEN(TokenAuthenticationArgs),
}

#[derive(Debug, PartialEq, Clone)]
pub struct PasswordAuthArgs {
    username: String,
    password: String,
    remember_me: bool,
}

#[derive(Debug, PartialEq, Clone)]
pub struct TokenAuthenticationArgs {
    account_token: String,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Authenticate {
    grant_type: GrantType,
}

impl Authenticate {
    pub fn new(grant_type: GrantType) -> Self {
        Self { grant_type }
    }

    pub fn grant_type(&self) -> &GrantType {
        &self.grant_type
    }
}

impl TokenAuthenticationArgs {
    pub fn new(token: &str) -> Self {
        Self {
            account_token: String::from(token),
        }
    }

    pub fn token(&self) -> &str {
        &self.account_token
    }
}

impl PasswordAuthArgs {
    pub fn new(username: &str, password: &str, remember_me: Option<bool>) -> Self {
        Self {
            username: String::from(username),
            password: String::from(password),
            remember_me: remember_me.unwrap_or(false),
        }
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn password(&self) -> &str {
        &self.password
    }
}

impl Frame for PasswordAuthArgs {
    fn decode(buffer: &mut Cursor<&[u8]>) -> Result<Self, FrameDecodeError>
    where
        Self: Sized,
    {
        assert_connection_type(&get_u16(buffer)?, &PASSWORD_AUTHENTICATION)?;

        let username = buffer.read_u32_str()?;
        let password = buffer.read_u32_str()?;
        let remember_me = buffer.get_u8() == 1;

        Ok(Self {
            username,
            password,
            remember_me,
        })
    }

    fn encode(&self) -> Vec<u8> {
        let mut buffer = Vec::new();
        buffer.put_u16(PASSWORD_AUTHENTICATION);
        buffer.put_u32_sized_str(&self.username);
        buffer.put_u32_sized_str(&self.password);
        buffer.put_u8(if self.remember_me { 1 } else { 0 });

        buffer
    }
}

impl Frame for TokenAuthenticationArgs {
    fn decode(buffer: &mut Cursor<&[u8]>) -> Result<Self, FrameDecodeError>
    where
        Self: Sized,
    {
        assert_connection_type(&get_u16(buffer)?, &AUTH_TOKEN_AUTHENTICATION)?;

        let token = buffer.read_u32_str()?;
        Ok(Self {
            account_token: token,
        })
    }

    fn encode(&self) -> Vec<u8> {
        let mut buffer = Vec::new();
        buffer.put_u16(AUTH_TOKEN_AUTHENTICATION);
        buffer.put_u32_sized_str(&self.account_token);

        buffer
    }
}

impl Frame for Authenticate {
    fn decode(buffer: &mut Cursor<&[u8]>) -> Result<Authenticate, FrameDecodeError> {
        assert_connection_type(&get_u16(buffer)?, &AUTHENTICATE)?;
        let grant_type_buf = [buffer.chunk()[0], buffer.chunk()[1]];
        let raw_grant_type = u16::from_be_bytes(grant_type_buf);

        let grant_type = match raw_grant_type {
            PASSWORD_AUTHENTICATION => GrantType::PASSWORD(PasswordAuthArgs::decode(buffer)?),
            AUTH_TOKEN_AUTHENTICATION => GrantType::TOKEN(TokenAuthenticationArgs::decode(buffer)?),
            actual => {
                return Err(FrameDecodeError::UnexpectedFrameType(actual));
            }
        };

        Ok(Self { grant_type })
    }

    fn encode(&self) -> Vec<u8> {
        let mut buffer = Vec::new();
        let grant_type_buff = match &self.grant_type {
            GrantType::PASSWORD(data) => data.encode(),
            GrantType::TOKEN(data) => data.encode(),
        };

        buffer.put_u16(AUTHENTICATE);
        buffer.put_slice(&grant_type_buff);

        buffer
    }
}

impl From<PasswordAuthArgs> for GrantType {
    fn from(value: PasswordAuthArgs) -> Self {
        Self::PASSWORD(value)
    }
}

impl From<TokenAuthenticationArgs> for GrantType {
    fn from(value: TokenAuthenticationArgs) -> Self {
        Self::TOKEN(value)
    }
}

#[cfg(test)]
pub mod tests {}
