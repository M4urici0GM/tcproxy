use std::io::Cursor;
use bytes::{BytesMut, Buf};
use tokio::{net::tcp::OwnedReadHalf, io::AsyncReadExt};
use tracing::{debug, trace};

use crate::{Result, FrameError};
use super::TcpFrame;

pub struct FrameReader<'a> {
  buffer: BytesMut,
  reader: &'a mut OwnedReadHalf,
}

impl<'a> FrameReader<'a> {
  pub fn new(reader: &'a mut OwnedReadHalf) -> Self {
    Self {
      reader,
      buffer: BytesMut::with_capacity(1024 * 8),
    }
  }

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

  pub async fn receive_frame(&mut self) -> Result<Option<TcpFrame>> {
      loop {
          if let Some(frame) = self.parse_frame().await? {
              return Ok(Some(frame));
          }

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