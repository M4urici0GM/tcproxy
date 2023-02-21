use std::io::Cursor;
use bytes::BufMut;
use crate::{Frame, FrameDecodeError};
use crate::framing::frame_types::AUTHENTICATE;
use crate::framing::utils::assert_connection_type;
use crate::io::{get_buffer, get_u32, get_u8};

#[derive(Debug, PartialEq)]
pub struct Authenticate {
    account_id: String,
    account_token: String,
}

impl Authenticate {
    pub fn new(id: &str, token: &str) -> Self {
        Self {
            account_id: String::from(id),
            account_token: String::from(token),
        }
    }
}

impl Frame for Authenticate {
    fn decode(buffer: &mut Cursor<&[u8]>) -> Result<Authenticate, FrameDecodeError> {
        assert_connection_type(&get_u8(buffer)?, &AUTHENTICATE)?;

        let account_id_size = get_u32(buffer)?;
        if 0 >= account_id_size {
            return Err(FrameDecodeError::CorruptedFrame);
        }

        let account_id = String::from_utf8(get_buffer(buffer, account_id_size)?)?;

        let token_size = get_u32(buffer)?;
        if 0 >= token_size {
            return Err(FrameDecodeError::CorruptedFrame);
        }

        let token = String::from_utf8(get_buffer(buffer, token_size)?)?;

        Ok(Self {
            account_id,
            account_token: token,
        })
    }

    fn encode(&self) -> Vec<u8> {
        let mut buffer = Vec::new();

        buffer.put_u8(AUTHENTICATE);
        buffer.put_u32(self.account_id.len() as u32);
        buffer.put_slice(self.account_id.as_bytes());
        buffer.put_u32(self.account_token.len() as u32);
        buffer.put_slice(self.account_token.as_bytes());

        buffer
    }
}


#[cfg(test)]
pub mod tests {
    use std::io::Cursor;
    use bytes::BufMut;
    use crate::{Frame, FrameDecodeError, is_type};
    use crate::framing::Authenticate;
    use crate::framing::frame_types::AUTHENTICATE;

    #[test]
    pub fn should_be_able_to_encode() {
        // Arrange
        let id = "some-account-id";
        let token = "some-account-token";
        let frame = Authenticate::new(id, token);

        // Arrange
        let encoded = frame.encode();

        // Assert
        assert_eq!(encoded.len(), id.len() + token.len() + 1 + 8); // ID_SIZE + TOKEN_SIZE + FRAME_TYPE + 2x STRING SIZES
    }

    #[test]
    pub fn should_be_able_to_decode() {
        // Arrange
        let id = "some-account-id";
        let token = "some-account-token";
        let mut buffer = Vec::new();

        buffer.put_u8(AUTHENTICATE);
        buffer.put_u32(id.len() as u32);
        buffer.put_slice(id.as_bytes());
        buffer.put_u32(token.len() as u32);
        buffer.put_slice(token.as_bytes());

        let mut cursor = Cursor::new(&buffer[..]);

        // Act
        let frame = Authenticate::decode(&mut cursor).unwrap();

        // Assert
        assert_eq!(frame.account_token, token);
        assert_eq!(frame.account_id, id);
    }

    #[test]
    pub fn should_return_err_when_account_id_size_is_zero() {
        // Arrange
        let id = "some-account-id";
        let token = "some-account-token";
        let mut buffer = Vec::new();

        buffer.put_u8(AUTHENTICATE);
        buffer.put_u32(0);
        buffer.put_slice(id.as_bytes());
        buffer.put_u32(token.len() as u32);
        buffer.put_slice(token.as_bytes());

        let mut cursor = Cursor::new(&buffer[..]);

        // Act
        let result = Authenticate::decode(&mut cursor);

        // Assert
        assert!(result.is_err());
        assert!(is_type!(result.unwrap_err(), FrameDecodeError::CorruptedFrame))
    }

    #[test]
    pub fn should_return_err_when_account_id_is_missing() {
        // Arrange
        let id = "some-account-id";
        let token = "some-account-token";
        let mut buffer = Vec::new();

        buffer.put_u8(AUTHENTICATE);
        buffer.put_u32(id.len() as u32);

        let mut cursor = Cursor::new(&buffer[..]);

        // Act
        let result = Authenticate::decode(&mut cursor);

        // Assert
        assert!(result.is_err());
        assert!(is_type!(result.unwrap_err(), FrameDecodeError::Incomplete))
    }

    #[test]
    pub fn should_return_err_when_account_token_size_is_zero() {
        // Arrange
        let id = "some-account-id";
        let token = "some-account-token";
        let mut buffer = Vec::new();

        buffer.put_u8(AUTHENTICATE);
        buffer.put_u32(id.len() as u32);
        buffer.put_slice(id.as_bytes());
        buffer.put_u32(0);
        buffer.put_slice(token.as_bytes());

        let mut cursor = Cursor::new(&buffer[..]);

        // Act
        let result = Authenticate::decode(&mut cursor);

        // Assert
        assert!(result.is_err());
        assert!(is_type!(result.unwrap_err(), FrameDecodeError::CorruptedFrame))
    }

    #[test]
    pub fn should_return_err_when_account_token_is_missing() {
        // Arrange
        let id = "some-account-id";
        let token = "some-account-token";
        let mut buffer = Vec::new();

        buffer.put_u8(AUTHENTICATE);
        buffer.put_u32(id.len() as u32);
        buffer.put_slice(id.as_bytes());
        buffer.put_u32(token.len() as u32);

        let mut cursor = Cursor::new(&buffer[..]);

        // Act
        let result = Authenticate::decode(&mut cursor);

        // Assert
        assert!(result.is_err());
        assert!(is_type!(result.unwrap_err(), FrameDecodeError::Incomplete))
    }
}