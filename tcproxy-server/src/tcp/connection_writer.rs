use std::net::SocketAddr;
use tokio::io::{AsyncWrite, AsyncWriteExt};
use tokio::sync::mpsc::Receiver;
use tracing::{error, trace};

use tcproxy_core::Result;

pub struct RemoteConnectionWriter<'a> {
    connection_addr: SocketAddr,
    receiver: Receiver<Vec<u8>>,
    writer: Box<dyn AsyncWrite + Unpin + Send + 'a>,
}

/// Writes buffers into remote connection.
impl<'a> RemoteConnectionWriter<'a> {
    pub fn new<T>(receiver: Receiver<Vec<u8>>, connection_addr: SocketAddr, writer: T) -> Self
    where
        T: AsyncWrite + Unpin + Send + 'a,
    {
        Self {
            receiver,
            connection_addr,
            writer: Box::new(writer),
        }
    }

    pub async fn start(&mut self) -> Result<()> {
        while let Some(buffer) = self.receiver.recv().await {
            match self.writer.write(&buffer).await {
                Ok(written) => {
                    trace!("written {} bytes to {}", written, self.connection_addr)
                }
                Err(err) => {
                    error!("failed to write into {}: {}", self.connection_addr, err);
                    break;
                }
            };

            let _ = self.writer.flush().await;
        }

        self.receiver.close();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{
        io::ErrorKind,
        net::Ipv4Addr,
        pin::Pin,
        task::{Context, Poll},
    };

    use super::*;
    use mockall::mock;
    use std::io;
    use std::io::Cursor;
    use std::net::IpAddr;
    use tokio::sync::mpsc;

    use crate::tests::utils::generate_random_buffer;

    mock! {
        pub Writer {}

        impl AsyncWrite for Writer {
            fn poll_write<'a>(mut self: Pin<&mut Self>, _cx: &mut Context<'a>, buf: &[u8]) -> Poll<io::Result<usize>>;
            fn poll_flush<'a>(self: Pin<&mut Self>, _cx: &mut Context<'a>) -> Poll<std::result::Result<(), io::Error>>;
            fn poll_shutdown<'a>(self: Pin<&mut Self>, _cx: &mut Context<'a>) -> Poll<std::result::Result<(), io::Error>>;
        }
    }

    #[tokio::test]
    async fn should_write_buffer_correctly() {
        let random_buffer = generate_random_buffer(1024);

        let mut bytes_buff: Vec<u8> = vec![];
        let cursor = Cursor::new(&mut bytes_buff);
        let (sender, receiver) = mpsc::channel::<Vec<u8>>(1);

        let addr = SocketAddr::new(IpAddr::from([127, 0, 0, 1]), 0);
        let mut connection_writer = RemoteConnectionWriter::new(receiver, addr, Box::new(cursor));

        let _ = sender.send(random_buffer[..].to_vec()).await;
        drop(sender);

        let result = connection_writer.start().await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn should_not_write() {
        let random_buffer = generate_random_buffer(1024 * 8);

        let ip = Ipv4Addr::new(127, 0, 0, 1);
        let addr = SocketAddr::new(std::net::IpAddr::V4(ip), 80);
        let (sender, receiver) = mpsc::channel::<Vec<u8>>(10);

        let mut mocked_stream = MockWriter::new();

        mocked_stream
            .expect_poll_write()
            .returning(|_, _| Poll::Ready(Err(std::io::Error::new(ErrorKind::Other, ""))));

        let mut connection_writer =
            RemoteConnectionWriter::new(receiver, addr, Box::new(mocked_stream));

        // Act

        let result = sender.send(random_buffer[..].to_vec()).await;
        assert!(result.is_ok());

        let result = connection_writer.start().await;

        // Assert
        assert!(result.is_ok());
        assert!(sender.is_closed());
    }
}
