#![allow(dead_code, unused_imports, unused_macros)]

use std::error::Error;
use std::io::Cursor;
use std::net::{IpAddr, SocketAddr};
use bytes::{Buf, BufMut, BytesMut};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use std::str::FromStr;
use std::time::Duration;
use rand::RngCore;
use tracing::debug;
use uuid::Bytes;

use tcproxy_server::{extract_enum_value, AppArguments, Server};
use tcproxy_core::{FrameError, TcpFrame};
use tcproxy_core::tcp::{TcpListener, SocketListener};


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
    let ping_frame = TcpFrame::Ping;
    let mut ping_buffer = ping_frame.to_buffer();

    let write_result = stream.write_buf(&mut ping_buffer).await;
    assert!(write_result.is_ok());

    let frame = receive_frame(&mut stream, &mut buffer).await;
    assert!(frame.is_ok());
    assert_eq!(frame.unwrap(), TcpFrame::Pong);
}

#[cfg(test)]
#[tokio::test]
async fn should_answer_client_connected() {
    let mut buffer = BytesMut::with_capacity(1024 * 8);

    let server = create_server().await;
    let result = TcpStream::connect(server).await;
    assert!(result.is_ok());

    let mut stream = result.unwrap();
    let ping_frame = TcpFrame::ClientConnected;
    let mut ping_buffer = ping_frame.to_buffer();

    let write_result = stream.write_buf(&mut ping_buffer).await;
    assert!(write_result.is_ok());

    let frame = receive_frame(&mut stream, &mut buffer).await;
    assert!(frame.is_ok());
    assert!(matches!(frame.unwrap(), TcpFrame::ClientConnectedAck { .. }));
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
    let ping_frame = TcpFrame::ClientConnected;
    let mut ping_buffer = ping_frame.to_buffer();

    let _ = stream.write_buf(&mut ping_buffer).await.unwrap();

    let frame = receive_frame(&mut stream, &mut buffer).await.unwrap();
    assert!(matches!(frame, TcpFrame::ClientConnectedAck {..}));

    let port = extract_enum_value!(frame, TcpFrame::ClientConnectedAck { port } => port);
    let ip = IpAddr::from_str("127.0.0.1").unwrap();
    let target_ip = SocketAddr::new(ip,port);

    let remote_stream = TcpStream::connect(target_ip).await;
    assert!(remote_stream.is_ok());

    let frame = receive_frame(&mut stream, &mut buffer).await;
    assert!(frame.is_ok());
    assert!(matches!(frame.unwrap(), TcpFrame::IncomingSocket {..}));
}

#[cfg(test)]
#[tokio::test]
async fn should_forward_data_successfully() {

    let mut buffer = BytesMut::with_capacity(1024 * 8);

    let server = create_server().await;
    let mut stream = TcpStream::connect(server).await.unwrap();
    write_tcp_frame(&mut stream, TcpFrame::ClientConnected).await;

    let frame = receive_frame(&mut stream, &mut buffer).await.unwrap();
    assert!(matches!(frame, TcpFrame::ClientConnectedAck {..}));

    let port = extract_enum_value!(frame, TcpFrame::ClientConnectedAck { port } => port);
    let ip = IpAddr::from_str("127.0.0.1").unwrap();
    let target_ip = SocketAddr::new(ip,port);

    let remote_stream = TcpStream::connect(target_ip).await;
    assert!(remote_stream.is_ok());

    let frame = receive_frame(&mut stream, &mut buffer).await;
    assert!(frame.is_ok());
    assert!(matches!(frame.unwrap(), TcpFrame::IncomingSocket {..}));

    let mut remote_stream = remote_stream.unwrap();
    let expected_buffer = generate_random_buffer(1024 * 2);

    let _ = remote_stream.write_all(&expected_buffer[..]).await.unwrap();

    let frame = receive_frame(&mut stream, &mut buffer).await.unwrap();
    let buffer = extract_enum_value!(frame, TcpFrame::DataPacketHost {
        buffer,
        buffer_size: _,
        connection_id: _
    } => buffer);

    assert_eq!(buffer.len(), expected_buffer.len());
    assert_eq!(&buffer[..], &expected_buffer[..]);
}


#[cfg(test)]
#[tokio::test]
async fn should_receive_data_successfully() -> Result<(), Box<dyn Error>> {
    let mut buffer = BytesMut::with_capacity(1024 * 8);

    let server = create_server().await;
    let mut stream = TcpStream::connect(server).await?;
    write_tcp_frame(&mut stream, TcpFrame::ClientConnected).await;

    let remote_port = extract_enum_value!(
        receive_frame(&mut stream, &mut buffer).await?,
        TcpFrame::ClientConnectedAck { port } => port
    );

    let ip = IpAddr::from_str("127.0.0.1").unwrap();
    let target_ip = SocketAddr::new(ip, remote_port);

    let mut remote_stream = TcpStream::connect(target_ip).await?;
    let connection_id = extract_enum_value!(
        read_frame(&mut stream, &mut buffer).await?,
        TcpFrame::IncomingSocket { connection_id } => connection_id);

    let expected_buffer = generate_random_buffer(1024 * 4);
    let frame = TcpFrame::DataPacketClient {
        connection_id,
        buffer: BytesMut::from(&expected_buffer[..]),
        buffer_size: expected_buffer.len() as u32,
    };

    write_tcp_frame(&mut stream, frame).await;

    let mut remote_buffer = BytesMut::with_capacity(1024 * 4);
    let bytes_read = remote_stream.read_buf(&mut remote_buffer).await?;

    assert_eq!(bytes_read, expected_buffer.len());
    assert_eq!(&expected_buffer[..], &remote_buffer[..bytes_read]);

    Ok(())
}

#[cfg(test)]
async fn write_tcp_frame(stream: &mut TcpStream, frame: TcpFrame) {
    let result = stream.write_all(&frame.to_buffer()).await;
    assert!(result.is_ok());
}

#[cfg(test)]
async fn receive_frame(stream: &mut TcpStream, buffer: &mut BytesMut) -> Result<TcpFrame, Box<dyn std::error::Error>> {
    tokio::select! {
        res = read_frame(stream, buffer) => res,
        _ = tokio::time::sleep(Duration::from_secs(30)) => Err("timeout reached".into()),
    }
}

#[cfg(test)]
async fn read_frame(stream: &mut TcpStream, buffer: &mut BytesMut) -> Result<TcpFrame, Box<dyn Error>> {
    loop {
        let bytes_read = match stream.read_buf(buffer).await {
            Ok(s) => s,
            Err(err) => return Err(err.into()),
        };

        if 0 == bytes_read {
            return Err("reached end of stream".into());
        }

        let mut cursor = Cursor::new(&buffer[..]);
        match TcpFrame::check(&mut cursor) {
            Ok(_) => {
                let position = cursor.position() as usize;
                cursor.set_position(0);

                let frame = TcpFrame::parse(&mut cursor)?;
                buffer.advance(position);

                return Ok(frame);
            }
            Err(FrameError::Incomplete) => {
                continue;
            }
            Err(err) => return Err(err.into()),
        }
    }
}

async fn create_server() -> SocketAddr {
    let args = AppArguments::new(0, "127.0.0.1", "1000:2000");
    let listener = TcpListener::bind(args.get_socket_addr()).await.unwrap();
    let listen_ip = listener.listen_ip().unwrap();

    let mut server = Server::new(args, Box::new(listener));

    tokio::spawn(async move {
        let result = server.run(tokio::signal::ctrl_c()).await;
        println!("server sopped with result. {:?}", result);
    });

    listen_ip
}

pub fn generate_random_buffer(buffer_size: i32) -> BytesMut {
    let mut buffer = BytesMut::with_capacity(buffer_size as usize);
  
    (0..buffer_size)
        .for_each(|_| {
            let random = rand::random::<u8>();
            buffer.put_u8(random);
        });
  
    return buffer;
  }