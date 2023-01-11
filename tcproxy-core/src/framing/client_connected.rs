use std::io::Cursor;
use crate::{Frame, FrameDecodeError};
use crate::framing::frame_types::CLIENT_CONNECTED;
use crate::framing::utils::assert_connection_type;
use crate::io::get_u8;

#[derive(Debug, PartialEq, Eq)]
pub struct ClientConnected;

impl ClientConnected {
    pub fn new() -> Self {
        Self {}
    }
}

impl Frame for ClientConnected {
    fn decode(buffer: &mut Cursor<&[u8]>) -> Result<Self, FrameDecodeError> where Self: Sized {
        assert_connection_type(&get_u8(buffer)?, &CLIENT_CONNECTED)?;
        return Ok(Self);
    }

    fn encode(&self) -> Vec<u8> {
        return vec![CLIENT_CONNECTED];
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;
    use crate::tcp_frame::Frame;
    use crate::framing::ClientConnected;
    use crate::framing::frame_types::CLIENT_CONNECTED;

    #[test]
    pub fn should_parse_client_connected() {
        // Arrange
        let buffer = vec![CLIENT_CONNECTED];
        let mut cursor = Cursor::new(&buffer[..]);

        // Act
        let frame = ClientConnected::decode(&mut cursor).unwrap();

        // Assert
        assert_eq!(ClientConnected, frame);
    }

    #[test]
    pub fn should_encode_client_connected() {
        // Arrange
        let frame = ClientConnected::new();

        // Act
        let result = frame.encode();

        // Assert
        assert_eq!(&vec![CLIENT_CONNECTED], &result[..]);
    }
}