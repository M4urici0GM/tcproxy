use std::io::Cursor;
use bytes::BufMut;
use crate::auth::token_handler::AuthToken;
use crate::{Frame, FrameDecodeError, PutU32String};
use crate::framing::frame_types::AUTHENTICATE_ACK;
use crate::framing::utils::assert_connection_type;
use crate::io::{get_u32_string, get_u16};
use std::fmt::{Display, Formatter};

#[derive(Debug, PartialEq, Default, Clone)]
pub struct AuthenticateAck {
    account_id: String,
    email: String,
    token: String,
}

impl AuthenticateAck {
    pub fn new(id: &str, email: &str, token: Option<AuthToken>) -> Self {
        Self {
            account_id: String::from(id),
            email: String::from(email),
            token: match token {
                Some(t) => String::from(t.get()),
                None => String::default(),
            },
        }
    }

    pub fn token(&self) -> &str {
        &self.token
    }
}

impl Frame for AuthenticateAck {
    fn decode(buffer: &mut Cursor<&[u8]>) -> Result<AuthenticateAck, FrameDecodeError> {
        assert_connection_type(&get_u16(buffer)?, &AUTHENTICATE_ACK)?;

        let account_id = get_u32_string(buffer)?;
        let email = get_u32_string(buffer)?;
        let token = get_u32_string(buffer)?;

        Ok(Self {
            account_id,
            email,
            token
        })
    }

    fn encode(&self) -> Vec<u8> {
        let mut buffer = Vec::new();

        buffer.put_u16(AUTHENTICATE_ACK);
        buffer.put_u32_sized_str(&self.account_id);
        buffer.put_u32_sized_str(&self.email);
        buffer.put_u32_sized_str(&self.token);

        buffer
    }
}

impl Display for AuthenticateAck {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "AuthenticateACK details: [email = {}, account-id = {}, token = {}]", self.email, self.account_id, self.token)
    }
}

#[cfg(test)]
pub mod tests {
    use std::io::Cursor;
    use bytes::BufMut;
    use crate::auth::token_handler::AuthToken;
    use crate::{Frame, FrameDecodeError, is_type};
    use crate::framing::AuthenticateAck;
    use crate::framing::frame_types::AUTHENTICATE_ACK;

    #[test]
    pub fn should_be_able_to_encode() {
        // Arrange
        let account_id = "account_id";
        let email = "some_email@gmail.com";
        let token = "some_token";
        let frame = AuthenticateAck::new(account_id, email, Some(AuthToken::new(token)));

        // Act
        let encoded = frame.encode();

        println!("{:?}", encoded);

        // Assert
        assert_eq!(encoded.len(), account_id.len() + token.len() + email.len() + std::mem::size_of::<u16>() + 12); // ID_SIZE + EMAIL_SIZE + FRAME_TYPE + 3x STRING SIZES
    }

    #[test]
    pub fn should_be_able_when_token_is_not_present() {
        // Arrange
        let account_id = "account_id";
        let email = "some_email@gmail.com";
        let token = String::default();
        let frame = AuthenticateAck::new(account_id, email, None);

        // Act
        let encoded = frame.encode();

        println!("{:?}", encoded);

        // Assert
        assert_eq!(encoded.len(), account_id.len() + token.len() + email.len() + std::mem::size_of::<u16>() + 12); // ID_SIZE + EMAIL_SIZE + FRAME_TYPE + 3x STRING SIZES
    }

    #[test]
    pub fn should_be_able_to_decode() {
        // Arrange
        let id = "some-account-id";
        let email = "some_email@gmail.com";
        let token = "some_token";
        let mut buffer = Vec::new();

        buffer.put_u16(AUTHENTICATE_ACK);
        buffer.put_u32(id.len() as u32);
        buffer.put_slice(id.as_bytes());
        buffer.put_u32(email.len() as u32);
        buffer.put_slice(email.as_bytes());
        buffer.put_u32(token.len() as u32);
        buffer.put_slice(token.as_bytes());

        let mut cursor = Cursor::new(&buffer[..]);

        // Act
        let frame = AuthenticateAck::decode(&mut cursor).unwrap();

        // Assert
        assert_eq!(frame.email, email);
        assert_eq!(frame.account_id, id);
        assert_eq!(frame.token, token);
    }

    #[test]
    pub fn should_return_incomplete_error_when_id_is_not_complete() {
        // Arrange
        let id = "some-account-id";
        let mut buffer = Vec::new();

        buffer.put_u16(AUTHENTICATE_ACK);
        buffer.put_u32(id.len() as u32);


        let mut cursor = Cursor::new(&buffer[..]);

        // Act
        let result = AuthenticateAck::decode(&mut cursor);

        // Assert
        assert!(result.is_err());
        assert!(is_type!(result.unwrap_err(), FrameDecodeError::Incomplete))
    }

    #[test]
    pub fn should_return_incomplete_error_when_email_is_not_complete() {
        // Arrange
        let id = "some-account-id";
        let email = "some_email@gmail.com";
        let mut buffer = Vec::new();

        buffer.put_u16(AUTHENTICATE_ACK);
        buffer.put_u32(id.len() as u32);
        buffer.put_slice(id.as_bytes());
        buffer.put_u32(email.len() as u32);

        let mut cursor = Cursor::new(&buffer[..]);

        // Act
        let result = AuthenticateAck::decode(&mut cursor);

        // Assert
        assert!(result.is_err());
        assert!(is_type!(result.unwrap_err(), FrameDecodeError::Incomplete))
    }
}