pub mod reader;
pub mod writer;

pub use reader::*;
pub use writer::*;

use crate::{tcp::SocketConnection, Result, TcpFrame};

/// represents TcpFrame buffer transport reader.
/// reads and writes TcpFrames from/info underlying buffer.
pub struct TcpFrameTransport {
    reader: DefaultTransportReader,
    writer: TransportWriter,
}

impl TcpFrameTransport {
    /// creates new instance of TcpFrameTransport.
    pub fn new<T>(connection: T) -> Self
    where
        T: SocketConnection,
    {
        let (reader, writer) = connection.split();
        Self {
            writer: TransportWriter::new(writer),
                reader: DefaultTransportReader::new(reader, 1024 * 8),
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
    pub fn split(self) -> (DefaultTransportReader, TransportWriter) {
        (self.reader, self.writer)
    }
}
