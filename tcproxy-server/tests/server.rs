#![allow(dead_code, unused_imports, unused_macros)]

use bytes::{Buf, BufMut, BytesMut};
use rand::RngCore;
use std::error::Error;
use std::io::Cursor;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tracing::debug;
use uuid::Bytes;

use tcproxy_core::tcp::{SocketListener, TcpListener};
use tcproxy_core::{FrameDecodeError, TcpFrame};
use tcproxy_core::framing::{ClientConnected, DataPacket, Ping};
use tcproxy_server::{extract_enum_value, AppArguments, Server, ServerConfig};
use tcproxy_server::managers::DefaultFeatureManager;

#[cfg(test)]
#[tokio::test]
async fn should_be_listening() {
    let server = create_server().await;
    println!("connecting to {}", server);

    let result = TcpStream::connect(server).await;
    assert!(result.is_ok());
}

#[cfg(test)]
#[tokio::test]
async fn should_answer_ping() {
    let mut buffer = BytesMut::with_capacity(1024 * 8);

    let server = create_server().await;
    let result = TcpStream::connect(server).await;
    assert!(result.is_ok());

    let mut stream = result.unwrap();
    let ping_frame = TcpFrame::Ping(Ping::new());
    let mut ping_buffer = TcpFrame::to_buffer(&ping_frame);

    let write_result = stream.write_buf(&mut ping_buffer).await;
    assert!(write_result.is_ok());

    let frame = receive_frame(&mut stream, &mut buffer).await;
    assert!(frame.is_ok());
}

#[cfg(test)]
#[tokio::test]
async fn should_answer_client_connected() {
    let mut buffer = BytesMut::with_capacity(1024 * 8);

    let server = create_server().await;
    let result = TcpStream::connect(server).await;
    assert!(result.is_ok());

    let mut stream = result.unwrap();
    let ping_frame = TcpFrame::ClientConnected(ClientConnected::new());
    let mut ping_buffer = TcpFrame::to_buffer(&ping_frame);

    let write_result = stream.write_buf(&mut ping_buffer).await;
    assert!(write_result.is_ok());

    let frame = receive_frame(&mut stream, &mut buffer).await;
    assert!(frame.is_ok());
    assert!(matches!(
        frame.unwrap(),
        TcpFrame::ClientConnectedAck { .. }
    ));
}

#[cfg(test)]
#[tokio::test]
async fn should_listen_in_ack_port() {
    use tcproxy_server::extract_enum_value;

    let mut buffer = BytesMut::with_capacity(1024 * 8);
    let server = create_server().await;
    let result = TcpStream::connect(server).await;
    assert!(result.is_ok());

    let mut stream = result.unwrap();
    let ping_frame = TcpFrame::ClientConnected(ClientConnected);
    let mut ping_buffer = TcpFrame::to_buffer(&ping_frame);

    let _ = stream.write_buf(&mut ping_buffer).await.unwrap();

    let frame = receive_frame(&mut stream, &mut buffer).await.unwrap();
    assert!(matches!(frame, TcpFrame::ClientConnectedAck { .. }));

    let port = extract_enum_value!(frame, TcpFrame::ClientConnectedAck(data) => *data.port());

    let ip = IpAddr::from_str("127.0.0.1").unwrap();
    let target_ip = SocketAddr::new(ip, port);

    let remote_stream = TcpStream::connect(target_ip).await;
    assert!(remote_stream.is_ok());

    let frame = receive_frame(&mut stream, &mut buffer).await;
    assert!(frame.is_ok());
    assert!(matches!(frame.unwrap(), TcpFrame::SocketConnected { .. }));
}

#[cfg(test)]
#[tokio::test]
async fn should_forward_data_successfully() {
    let mut buffer = BytesMut::with_capacity(1024 * 8);

    let server = create_server().await;
    let mut stream = TcpStream::connect(server).await.unwrap();
    write_tcp_frame(&mut stream, TcpFrame::ClientConnected(ClientConnected)).await;

    let frame = receive_frame(&mut stream, &mut buffer).await.unwrap();
    assert!(matches!(frame, TcpFrame::ClientConnectedAck { .. }));

    let port = extract_enum_value!(frame, TcpFrame::ClientConnectedAck(data) => *data.port());
    let ip = IpAddr::from_str("127.0.0.1").unwrap();
    let target_ip = SocketAddr::new(ip, port);

    let remote_stream = TcpStream::connect(target_ip).await;
    assert!(remote_stream.is_ok());

    let frame = receive_frame(&mut stream, &mut buffer).await;
    assert!(frame.is_ok());
    assert!(matches!(frame.unwrap(), TcpFrame::SocketConnected { .. }));

    let mut remote_stream = remote_stream.unwrap();
    let expected_buffer = generate_random_buffer(1024 * 2);

    remote_stream.write_all(&expected_buffer[..]).await.unwrap();

    let buffer = match receive_frame(&mut stream, &mut buffer).await.unwrap() {
        TcpFrame::DataPacket(data) => data.buffer().to_owned(),
        value => {
            panic!("didn't expected value {value}");
        }
    };

    assert_eq!(buffer.len(), expected_buffer.len());
    assert_eq!(&buffer[..], &expected_buffer[..]);
}

#[cfg(test)]
#[tokio::test]
async fn should_receive_data_successfully() -> Result<(), Box<dyn Error>> {
    let mut buffer = BytesMut::with_capacity(1024 * 8);

    let server = create_server().await;
    let mut stream = TcpStream::connect(server).await?;
    write_tcp_frame(&mut stream, TcpFrame::ClientConnected(ClientConnected)).await;

    let remote_port = extract_enum_value!(
        receive_frame(&mut stream, &mut buffer).await?,
        TcpFrame::ClientConnectedAck(data) => *data.port()
    );

    let ip = IpAddr::from_str("127.0.0.1").unwrap();
    let target_ip = SocketAddr::new(ip, remote_port);

    let mut remote_stream = TcpStream::connect(target_ip).await?;
    let connection_id = extract_enum_value!(
        read_frame(&mut stream, &mut buffer).await?,
        TcpFrame::SocketConnected(data) => *data.connection_id());

    let expected_buffer = generate_random_buffer(1024 * 4);
    let frame = TcpFrame::DataPacket(DataPacket::new(&connection_id, &expected_buffer));

    write_tcp_frame(&mut stream, frame).await;

    let mut remote_buffer = BytesMut::with_capacity(1024 * 4);
    let bytes_read = remote_stream.read_buf(&mut remote_buffer).await?;

    assert_eq!(bytes_read, expected_buffer.len());
    assert_eq!(&expected_buffer[..], &remote_buffer[..bytes_read]);

    Ok(())
}

#[cfg(test)]
async fn write_tcp_frame(stream: &mut TcpStream, frame: TcpFrame) {
    let buffer = TcpFrame::to_buffer(&frame);
    let result = stream.write_all(&buffer).await;
    assert!(result.is_ok());
}

#[cfg(test)]
async fn receive_frame(
    stream: &mut TcpStream,
    buffer: &mut BytesMut,
) -> Result<TcpFrame, Box<dyn Error>> {
    tokio::select! {
        res = read_frame(stream, buffer) => res,
        _ = tokio::time::sleep(Duration::from_secs(30)) => Err("timeout reached".into()),
    }
}

#[cfg(test)]
async fn read_frame(
    stream: &mut TcpStream,
    buffer: &mut BytesMut,
) -> Result<TcpFrame, Box<dyn Error>> {
    loop {
        let bytes_read = match stream.read_buf(buffer).await {
            Ok(s) => s,
            Err(err) => return Err(err.into()),
        };

        if 0 == bytes_read {
            return Err("reached end of stream".into());
        }

        let mut cursor = Cursor::new(&buffer[..]);
        match TcpFrame::parse(&mut cursor) {
            Ok(frame) => {
                buffer.advance(cursor.position() as usize);
                return Ok(frame);
            }
            Err(FrameDecodeError::Incomplete) => {
                continue;
            }
            Err(err) => {
                return Err(err.into())
            },
        }
    }
}

async fn create_server() -> SocketAddr {
    let ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
    let socket_addr = SocketAddr::new(ip, 0);

    let listener = TcpListener::bind(socket_addr).await.unwrap();
    let listen_ip = listener.listen_ip().unwrap();
    let server_config = ServerConfig::new(
        11000,
        15000,
        ip,
        0,
        "proxy.server.local",
        120);

    let feature_manager = DefaultFeatureManager::new(server_config);
    let mut server = Server::new(feature_manager, listener);

    tokio::spawn(async move {
        let result = server.run(tokio::signal::ctrl_c()).await;
        println!("server sopped with result. {:?}", result);
    });

    listen_ip
}

pub fn generate_random_buffer(buffer_size: i32) -> BytesMut {
    let mut buffer = BytesMut::with_capacity(buffer_size as usize);

    (0..buffer_size).for_each(|_| {
        let random = rand::random::<u8>();
        buffer.put_u8(random);
    });

    buffer
}
