pub mod reader;
pub mod writer;

use std::net::SocketAddr;
use tokio::net::TcpStream as TokioTcpStream;
use tokio_native_tls::native_tls::{TlsConnector};
use tokio_native_tls::TlsConnector as TokioTlsConnector;
use tracing::{debug, error};

pub use reader::*;
pub use writer::*;

use crate::stream::Stream;
use crate::{Result, TcpFrame};

/// represents TcpFrame buffer transport reader.
/// reads and writes TcpFrames from/info underlying buffer.
pub struct TcpFrameTransport {
    reader: TransportReader,
    writer: TransportWriter,
}

impl TcpFrameTransport {
    /// creates new instance of TcpFrameTransport.
    pub fn new(connection: Stream) -> Self {
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

    pub async fn connect(addr: SocketAddr, tls: bool) -> Result<TcpFrameTransport> {
        match TokioTcpStream::connect(addr).await {
            Ok(stream) => {
                debug!("Connected to server..");
                let stream = match tls {
                    false => Stream::new(stream),
                    true => {
                        let connector = match TlsConnector::builder().build() {
                            Ok(c) => c,
                            Err(err) => {
                                tracing::error!("error when trying to create TlsAcceptor: {}", err);
                                return Err(err.into());
                            }
                        };

                        let connector = TokioTlsConnector::from(connector);
                        let stream = connector.connect("127.0.0.1", stream).await?;
                        tracing::debug!("successfully made TLS handshake! :rocket:");

                        Stream::new(stream)
                    }
                };

                Ok(Self::new(stream))
            }
            Err(err) => {
                error!("Failed to connect to server. Check you network connection and try again.");
                Err(format!("Failed when connecting to server: {}", err).into())
            }
        }
    }

    /// sends a TcpFrame to underlying tcp-stream, and grabs the first incoming tcp-frame
    /// caller must handler if received frame is the expected
    /// note that this method should only be used when there's no active waiting for another frame.
    pub async fn send_frame(&mut self, frame: &TcpFrame) -> Result<TcpFrame> {
        self.write(frame.clone()).await?;
        match self.reader.next().await? {
            Some(f) => Ok(f),
            None => {
                debug!("received none. it means the server closed the connection.");
                Err("failed to do handshake with server.".into())
            }
        }
    }
}
