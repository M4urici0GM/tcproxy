use bytes::BytesMut;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::tcp::{ReadHalf, WriteHalf};
use tokio::net::TcpStream;
use tracing::{debug, error};

use crate::{Result};

pub struct DuplexTcpStream<'a> {
    left: &'a mut TcpStream,
    right: &'a mut TcpStream,
    buffer_size: usize,
}

impl<'a> DuplexTcpStream<'a> {
    pub fn join(left_stream: &'a mut TcpStream, right_stream: &'a mut TcpStream, buffer_size: Option<usize>) -> Self {
        let buffer_size = match buffer_size {
            Some(size) => size,
            None => 1024 * 8,
        };

        Self {
            left: left_stream,
            right: right_stream,
            buffer_size,
        }
    }

    pub async fn start(&'a mut self) -> Result<()> {
        let (mut source_reader, mut source_writer) = self.left.split();
        let (mut target_reader, mut target_writer) = self.right.split();

        let source_to_target = Self::start_streaming(self.buffer_size.clone(), &mut source_reader, &mut target_writer);
        let target_to_source = Self::start_streaming(self.buffer_size.clone(), &mut target_reader, &mut source_writer);

        return tokio::select! {
            stt_result = source_to_target => stt_result,
            tts_result = target_to_source => tts_result,
        };
    }

    async fn start_streaming(buffer_size: usize, reader: &mut ReadHalf<'a>, writer: &mut WriteHalf<'a>) -> Result<()> {
        let mut buffer = BytesMut::with_capacity(buffer_size);

        loop {
            let bytes_read = match reader.read_buf(&mut buffer).await {
                Ok(size) => size,
                Err(err) => {
                    error!("Error when reading from stream. {}", err);
                    return Err(err.into());
                }
            };


            if 0 == bytes_read {
                return Ok(());
            }

            debug!("Read {} bytes from stream", bytes_read);
            buffer.truncate(bytes_read);
            let bytes_written = match writer.write_buf(&mut buffer).await {
                Ok(size) => size,
                Err(err) => {
                    error!("Error when writing to stream..");
                    return Err(err.into());
                }
            };

            let _ = writer.flush().await;

            debug!("Written {} bytes into stream", bytes_written);
            if bytes_read != bytes_written {
                return Err("Buffer mismatch.. Closing..".into());
            }
        }
    }
}