use bytes::{Buf, BytesMut};
use std::io::Cursor;
use async_trait::async_trait;
use tokio::{
    io::{AsyncRead, AsyncReadExt},
};
use tracing::{debug, trace};
use mockall::automock;

use crate::{FrameError, Result, TcpFrame};

/// represents TcpFrame transport reader
/// read new frames from underlying buffer.
pub struct DefaultTransportReader {
    buffer: BytesMut,
    reader: Box<dyn AsyncRead + Send + Unpin>,
}

#[automock]
#[async_trait]
pub trait TransportReader: Send {
    async fn next(&mut self) -> Result<Option<TcpFrame>>;
}

#[async_trait]
impl TransportReader for DefaultTransportReader {
    async fn next(&mut self) -> Result<Option<TcpFrame>> {
        loop {
            if let Some(frame) = self.parse_frame().await? {
                return Ok(Some(frame));
            }

            // when we read 0 from socket. it could mean two things.
            // if the buffer is empty, it means that there's no data left to sent,
            //   and the client disconnected gracefully.
            // if the buffer is not empty, it means the socket closed before sending all required data,
            // so probably socket was reset by peer.
            if 0 == self.reader.read_buf(&mut self.buffer).await? {
                trace!("read 0 bytes from client.");
                if self.buffer.is_empty() {
                    debug!("received 0 bytes from client, and buffer is empty.");
                    return Ok(None);
                }

                return Err("connection reset by peer.".into());
            }
        }
    }
}

impl DefaultTransportReader {
    pub fn new<T>(reader: T, buffer_size: usize) -> Self
    where
        T: AsyncRead  + Send + Unpin + 'static,
    {
        Self {
            reader: Box::new(reader),
            buffer: BytesMut::with_capacity(buffer_size),
        }
    }

    /// checks if underling buffer has new frame available.
    /// if does, it will parse and return available frame.
    async fn parse_frame(&mut self) -> Result<Option<TcpFrame>> {
        let mut cursor = Cursor::new(&self.buffer[..]);
        match TcpFrame::check(&mut cursor) {
            Ok(_) => {
                let position = cursor.position() as usize;
                cursor.set_position(0);

                let frame = TcpFrame::parse(&mut cursor)?;
                self.buffer.advance(position);

                Ok(Some(frame))
            }
            Err(FrameError::Incomplete) => Ok(None),
            Err(err) => Err(err.into()),
        }
    }
}
