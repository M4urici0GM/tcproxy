pub mod writer;
pub mod reader;

pub use writer::*;
pub use reader::*;

use crate::{Result, TcpFrame};
use tokio::net::TcpStream;

/// represents TcpFrame buffer transport reader.
/// reads and writes TcpFrames from/info underlying buffer.
pub struct TcpFrameTransport {
  reader: TransportReader,
  writer: TransportWriter,
}

impl TcpFrameTransport {
  /// creates new instance of TcpFrameTransport.
  pub fn new(connection: TcpStream) -> Self {
      let (reader, writer) = connection.into_split();
      Self {
          writer: TransportWriter::new(writer),
          reader: TransportReader::new(reader, 1024 * 8),
      }
  }

  /// fetches new tcpframe from underlying reader.
  pub async fn next(&mut self) -> Result<Option<TcpFrame>> {
      self.reader.next().await
  }

  /// writes new tcpframe to underlying writer.
  pub async fn write(&mut self, frame: TcpFrame) -> Result<()> {
      self.writer.send(frame).await
  }
  
  /// splits TcpFrameTransport into its reader and writer.
  pub fn split(self) -> (TransportReader, TransportWriter) {
      (self.reader, self.writer)
  }
}
