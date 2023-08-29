

use tokio::net::TcpStream;
use tokio_native_tls::TlsAcceptor;

use crate::stream::Stream;

pub async fn accept_tls(stream: TcpStream, acceptor: &TlsAcceptor) -> crate::Result<Stream> {
    let stream = acceptor.accept(stream).await?;
    Ok(Stream::new(stream))
}
