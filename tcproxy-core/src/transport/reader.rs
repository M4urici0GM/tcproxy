use bytes::{Buf, BytesMut};
use std::io::Cursor;
use tokio::io::{AsyncRead, AsyncReadExt};
use tracing::{debug, error, trace};

use crate::{FrameDecodeError, Result, TcpFrame};

/// represents TcpFrame transport reader
/// read new frames from underlying buffer.
pub struct TransportReader {
    buffer: BytesMut,
    reader: Box<dyn AsyncRead + Send + Unpin>,
}

impl TransportReader {
    pub fn new<T>(reader: T, buffer_size: usize) -> Self
    where
        T: AsyncRead + Send + Unpin + 'static,
    {
        Self {
            reader: Box::new(reader),
            buffer: BytesMut::with_capacity(buffer_size),
        }
    }

    /// Tries to fetch next frame from underlying stream
    /// If returns None, means that the connection was closed and no more bytes will be sent.
    /// Maybe TODO?: Add timeout feature, if after X tries or time closes the connection
    pub async fn next(&mut self) -> Result<Option<TcpFrame>> {
        loop {
            if let Some(frame) = self.probe_frame()? {
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

    /// Tries to parse a frame if present on underlying buffer
    /// If frame is yet not complete, it will return None, hoping that
    /// in the next iteration, the frame should be complete.
    fn probe_frame(&mut self) -> Result<Option<TcpFrame>> {
        let mut cursor = Cursor::new(&self.buffer[..]);
        match TcpFrame::parse(&mut cursor) {
            Ok(frame) => {
                trace!("found new frame on buffer: {}", frame);
                self.buffer.advance(cursor.position() as usize);
                Ok(Some(frame))
            }
            Err(FrameDecodeError::Incomplete) => {
                trace!("incomplete frame on buffer.. {}", self.buffer.len());
                Ok(None)
            }
            Err(err) => {
                error!("error trying to parse frame {}", err);
                Err(err.into())
            }
        }
    }
}
