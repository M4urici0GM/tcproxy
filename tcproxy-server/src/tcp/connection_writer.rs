use bytes::BytesMut;
use std::net::SocketAddr;
use tokio::io::{AsyncWrite, AsyncWriteExt};
use tokio::sync::mpsc::Receiver;
use tracing::{error, trace};

use tcproxy_core::Result;

pub struct RemoteConnectionWriter {
    connection_addr: SocketAddr,
    receiver: Receiver<BytesMut>,
}

impl RemoteConnectionWriter {
    pub fn new(receiver: Receiver<BytesMut>, connection_addr: SocketAddr) -> Self {
        Self {
            connection_addr,
            receiver,
        }
    }

    pub async fn start<T>(&mut self, mut writer: T) -> Result<()>
    where
        T: AsyncWrite + Unpin,
    {
        while let Some(mut buffer) = self.receiver.recv().await {
            let mut buffer = buffer.split();
            match writer.write_buf(&mut buffer).await {
                Ok(written) => {
                    trace!("written {} bytes to {}", written, self.connection_addr)
                }
                Err(err) => {
                    error!("failed to write into {}: {}", self.connection_addr, err);
                    break;
                }
            };

            let _ = writer.flush().await;
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
    use bytes::BytesMut;
    use mockall::mock;
    use std::io;
    use std::result;
    use tokio::sync::mpsc;

    use crate::tests::utils::generate_random_buffer;

    mock! {
        pub Writer {}

        impl AsyncWrite for Writer {
            fn poll_write<'a>(mut self: Pin<&mut Self>, _cx: &mut Context<'a>, buf: &[u8]) -> Poll<io::Result<usize>>;
            fn poll_flush<'a>(self: Pin<&mut Self>, _cx: &mut Context<'a>) -> Poll<result::Result<(), io::Error>>;
            fn poll_shutdown<'a>(self: Pin<&mut Self>, _cx: &mut Context<'a>) -> Poll<result::Result<(), io::Error>>;
        }
    }

    #[tokio::test]
    async fn should_write_buffer_correctly() {
        let random_buffer = generate_random_buffer(1024 * 8);

        let ip = Ipv4Addr::new(127, 0, 0, 1);
        let addr = SocketAddr::new(std::net::IpAddr::V4(ip), 80);
        let (sender, receiver) = mpsc::channel::<BytesMut>(10);

        let mut mocked_stream = MockWriter::new();
        let mut connection_writer = RemoteConnectionWriter::new(receiver, addr);

        let _ = sender.send(BytesMut::from(&random_buffer[..])).await;
        drop(sender);

        mocked_stream
            .expect_poll_write()
            .withf(move |_, buff| {
                buff.iter()
                    .enumerate()
                    .all(|(i, value)| random_buffer[i] == *value)
            })
            .returning(|_, _| Poll::Ready(Ok(0)));

        mocked_stream
            .expect_poll_flush()
            .times(1)
            .returning(|_| Poll::Ready(Ok(())));

        let result = connection_writer.start(Box::new(mocked_stream)).await;
        assert_eq!(true, result.is_ok());
    }

    #[tokio::test]
    async fn should_not_write() {
        let random_buffer = generate_random_buffer(1024 * 8);

        let ip = Ipv4Addr::new(127, 0, 0, 1);
        let addr = SocketAddr::new(std::net::IpAddr::V4(ip), 80);
        let (sender, receiver) = mpsc::channel::<BytesMut>(10);

        let mut mocked_stream = MockWriter::new();
        let mut connection_writer = RemoteConnectionWriter::new(receiver, addr);

        mocked_stream
            .expect_poll_write()
            .returning(|_, _| Poll::Ready(Err(std::io::Error::new(ErrorKind::Other, ""))));

        let result = sender.send(BytesMut::from(&random_buffer[..])).await;
        assert_eq!(true, result.is_ok());

        let result = connection_writer.start(Box::new(mocked_stream)).await;
        assert_eq!(true, result.is_ok());
        assert_eq!(true, sender.is_closed());
    }
}
