use std::io::Cursor;
use bytes::BufMut;
use crate::{Frame, FrameDecodeError, PutU32String};
use crate::framing::frame_types::AUTHENTICATE_ACK;
use crate::framing::utils::assert_connection_type;
use crate::io::{get_u32_string, get_u8};

#[derive(Debug, PartialEq, Default)]
pub struct AuthenticateAck {
    account_id: String,
    email: String,
}

impl AuthenticateAck {
    pub fn new(id: &str, email: &str) -> Self {
        Self {
            account_id: String::from(id),
            email: String::from(email),
        }
    }
}

impl Frame for AuthenticateAck {
    fn decode(buffer: &mut Cursor<&[u8]>) -> Result<AuthenticateAck, FrameDecodeError> {
        assert_connection_type(&get_u8(buffer)?, &AUTHENTICATE_ACK)?;

        let account_id = get_u32_string(buffer)?;
        let email = get_u32_string(buffer)?;

        Ok(Self {
            account_id,
            email
        })
    }

    fn encode(&self) -> Vec<u8> {
        let mut buffer = Vec::new();

        buffer.put_u8(AUTHENTICATE_ACK);
        buffer.put_u32_sized_str(&self.account_id);
        buffer.put_u32_sized_str(&self.email);

        buffer
    }
}

#[cfg(test)]
pub mod tests {
    use std::io::Cursor;
    use bytes::BufMut;
    use crate::{Frame, FrameDecodeError, is_type};
    use crate::framing::AuthenticateAck;
    use crate::framing::frame_types::AUTHENTICATE_ACK;

    #[test]
    pub fn should_be_able_to_encode() {
        // Arrange
        let account_id = "account_id";
        let email = "some_email@gmail.com";
        let frame = AuthenticateAck::new(account_id, email);

        // Act
        let encoded = frame.encode();

        // Assert
        assert_eq!(encoded.len(), account_id.len() + email.len() + 1 + 8); // ID_SIZE + EMAIL_SIZE + FRAME_TYPE + 2x STRING SIZES
    }

    #[test]
    pub fn should_be_able_to_decode() {
        // Arrange
        let id = "some-account-id";
        let email = "some_email@gmail.com";
        let mut buffer = Vec::new();

        buffer.put_u8(AUTHENTICATE_ACK);
        buffer.put_u32(id.len() as u32);
        buffer.put_slice(id.as_bytes());
        buffer.put_u32(email.len() as u32);
        buffer.put_slice(email.as_bytes());

        let mut cursor = Cursor::new(&buffer[..]);

        // Act
        let frame = AuthenticateAck::decode(&mut cursor).unwrap();

        // Assert
        assert_eq!(frame.email, email);
        assert_eq!(frame.account_id, id);
    }

    #[test]
    pub fn should_return_incomplete_error_when_id_is_not_complete() {
        // Arrange
        let id = "some-account-id";
        let email = "some_email@gmail.com";
        let mut buffer = Vec::new();

        buffer.put_u8(AUTHENTICATE_ACK);
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

        buffer.put_u8(AUTHENTICATE_ACK);
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