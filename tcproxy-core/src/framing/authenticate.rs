use std::io::Cursor;
use bytes::buf::BufMut;
use mongodb::bson::Uuid;
use crate::{Frame, FrameDecodeError, PutBsonUuid, PutU32String};
use crate::framing::frame_types::AUTHENTICATE;
use crate::framing::utils::assert_connection_type;
use crate::io::{get_u32_string, get_u8};
use crate::ReadBsonUuid;

#[derive(Debug, PartialEq)]
pub struct Authenticate {
    account_id: Uuid,
    account_token: String,
}

impl Authenticate {
    pub fn new(id: &Uuid, token: &str) -> Self {
        Self {
            account_id: *id,
            account_token: String::from(token),
        }
    }

    pub fn token(&self) -> &str { &self.account_token }

    pub fn account_id(&self) -> &Uuid { &self.account_id }
}

impl Frame for Authenticate {
    fn decode(buffer: &mut Cursor<&[u8]>) -> Result<Authenticate, FrameDecodeError> {
        assert_connection_type(&get_u8(buffer)?, &AUTHENTICATE)?;
        let account_id = buffer.read_bson_uuid()?;
        let account_token = get_u32_string(buffer)?;

        Ok(Self {
            account_id,
            account_token,
        })
    }

    fn encode(&self) -> Vec<u8> {
        let mut buffer = Vec::new();

        buffer.put_u8(AUTHENTICATE);
        buffer.put_bson_uuid(&self.account_id);
        buffer.put_u32_sized_str(&self.account_token);

        buffer
    }
}


#[cfg(test)]
pub mod tests {
    use std::io::Cursor;
    use bytes::BufMut;
    use mongodb::bson::Uuid;
    use crate::{Frame, FrameDecodeError, is_type};
    use crate::framing::Authenticate;
    use crate::framing::frame_types::AUTHENTICATE;

    #[test]
    pub fn should_be_able_to_encode() {
        // Arrange
        let id = Uuid::new();
        let token = "some-account-token";
        let frame = Authenticate::new(&id, token);

        // Arrange
        let encoded = frame.encode();

        // Assert
        assert_eq!(encoded.len(), 1 + 16 + 4 + token.len()); // ID_SIZE + TOKEN_SIZE + FRAME_TYPE + 2x STRING SIZES
    }

    #[test]
    pub fn should_be_able_to_decode() {
        // Arrange
        let id = Uuid::new();
        let token = "some-account-token";
        let mut buffer = Vec::new();

        buffer.put_u8(AUTHENTICATE);
        buffer.put_slice(&id.bytes());
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
    pub fn should_return_err_when_account_id_is_missing() {
        // Arrange
        let id = Uuid::new();
        let mut buffer = Vec::new();

        buffer.put_u8(AUTHENTICATE);
        buffer.put_u32(16u32);

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
        let id = Uuid::new();
        let token = "some-account-token";
        let mut buffer = Vec::new();

        buffer.put_u8(AUTHENTICATE);
        buffer.put_slice(&id.bytes());
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