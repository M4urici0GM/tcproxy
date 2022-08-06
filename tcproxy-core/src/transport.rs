use std::{io::Cursor, sync::Arc};
use bytes::{BytesMut, Buf};
use tokio::{net::{tcp::{OwnedReadHalf, OwnedWriteHalf}, TcpStream}, io::{AsyncReadExt, AsyncWriteExt}};
use tracing::{debug, trace};

use crate::{Result, FrameError};
use super::TcpFrame;

pub struct TcpFrameTransport {
  reader: TransportReader,
  writer: TransportWriter,
}

pub struct TransportWriter {
  writer: OwnedWriteHalf,
}

pub struct TransportReader {
  buffer: BytesMut,
  reader: OwnedReadHalf,
}

impl TransportReader {
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

  pub async fn next(&mut self) -> Result<Option<TcpFrame>> {
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

impl TransportWriter {
  pub async fn send(&mut self, frame: TcpFrame) -> Result<()> {
      let mut buffer = frame.to_buffer();
      let bytes_written =  self.writer.write_buf(&mut buffer).await?;
      trace!("written {} bytes to socket.", bytes_written);

      Ok(())
  }
}

impl TcpFrameTransport {
  pub fn new(connection: TcpStream) -> Self {
      let (reader, writer) = connection.into_split();
      Self {
          writer: TransportWriter { writer },
          reader: TransportReader {
            reader,
            buffer: BytesMut::with_capacity(1024 * 7),
          }
      }
  }

  pub async fn next(&mut self) -> Result<Option<TcpFrame>> {
    self.reader.next().await
  }

  pub fn split(self) -> (TransportReader, TransportWriter) {
      (self.reader, self.writer)
  }
}