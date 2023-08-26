use std::{
    cell::UnsafeCell,
    pin::Pin,
    sync::{
        atomic::{AtomicBool, Ordering::Acquire},
        Arc,
    },
    task::{Context, Poll},
};

use tokio::{
    io::{AsyncRead, AsyncWrite, ReadBuf},
    net::TcpStream,
};
use tokio_native_tls::TlsStream;

pub trait AsyncStream: AsyncRead + AsyncWrite + Unpin {}

impl AsyncStream for TcpStream {}

impl AsyncStream for TlsStream<TcpStream> {}

pub struct Stream {
    inner: Box<dyn AsyncStream>,
}

impl Stream {
    pub fn new<T: AsyncStream>(inner: T) -> Self {
        Self {
            inner: Box::new(inner),
        }
    }

    pub fn into_split(self) -> (OwnedReadHalf, OwnedWriteHalf) {
        let arc = Arc::new(Inner::new(self));

        let read_half = OwnedReadHalf {
            inner: Arc::clone(&arc),
        };
        let write_half = OwnedWriteHalf { inner: arc };

        (read_half, write_half)
    }
}

impl AsyncRead for Stream {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        Pin::new(&mut *self.inner).poll_read(cx, buf)
    }
}

impl AsyncWrite for Stream {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, std::io::Error>> {
        Pin::new(&mut *self.inner).poll_write(cx, buf)
    }

    fn poll_flush(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        Pin::new(&mut *self.inner).poll_flush(cx)
    }

    fn poll_shutdown(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        Pin::new(&mut *self.inner).poll_shutdown(cx)
    }
}

pub struct Inner {
    stream: UnsafeCell<Stream>,
    lock: AtomicBool,
}

pub struct InnerGuard<'a> {
    inner: &'a Inner,
}

impl Inner {
    pub fn new(stream: Stream) -> Self {
        Self {
            stream: UnsafeCell::new(stream),
            lock: AtomicBool::new(false),
        }
    }
}

impl Inner {
    pub(crate) fn poll_lock<'a>(&self, cx: &mut Context<'_>) -> Poll<InnerGuard<'a>> {
        if self
            .lock
            .compare_exchange(false, true, Acquire, Acquire)
            .is_ok()
        {
            return Poll::Ready(InnerGuard { inner: self });
        }

        std::thread::yield_now();
        cx.waker().wake_by_ref();

        Poll::Pending
    }
}

pub struct OwnedWriteHalf {
    inner: Arc<Inner>,
}

pub struct OwnedReadHalf {
    inner: Arc<Inner>,
}

unsafe impl Send for OwnedWriteHalf {}
unsafe impl Send for OwnedReadHalf {}
unsafe impl Sync for OwnedWriteHalf {}
unsafe impl Sync for OwnedReadHalf {}
