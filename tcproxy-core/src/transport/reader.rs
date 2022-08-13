use std::io::Cursor;
use bytes::{BytesMut, Buf};
use tokio::{io::AsyncReadExt, net::tcp::OwnedReadHalf};
use tracing::{debug, trace};

use crate::{Result, TcpFrame, FrameError};

/// represents TcpFrame transport reader
/// read new frames from underlying buffer.
pub struct TransportReader {
  buffer: BytesMut,
  reader: OwnedReadHalf,
}

impl TransportReader {
  pub fn new(reader: OwnedReadHalf, buffer_size: usize) -> Self {
    Self {
      reader,
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

  /// tries getting next frame from underling buffer.
  pub async fn next(&mut self) -> Result<Option<TcpFrame>> {
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