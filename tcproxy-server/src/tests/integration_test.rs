use std::io::Cursor;
use std::net::{IpAddr, SocketAddr};
use bytes::{Buf, BytesMut};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use std::str::FromStr;
use rand::RngCore;
use uuid::Bytes;

use tcproxy_core::{FrameError, TcpFrame};

use crate::{AppArguments, DefaultListener, Server, Listener};


macro_rules! extract_enum_value {
  ($value:expr, $pattern:pat => $extracted_value:expr) => {
    match $value {
      $pattern => $extracted_value,
      _ => panic!("Pattern doesn't match!"),
    }
  };
}

#[tokio::test]
async fn should_be_listening() {
    let server = create_server().await;
    println!("connecting to {}", server);

    let result = TcpStream::connect(server).await;
    assert!(result.is_ok());
}


#[tokio::test]
async fn should_answer_ping() {
    let server = create_server().await;
    let result = TcpStream::connect(server).await;
    assert!(result.is_ok());

    let mut stream = result.unwrap();
    let ping_frame = TcpFrame::Ping;
    let mut ping_buffer = ping_frame.to_buffer();

    let write_result = stream.write_buf(&mut ping_buffer).await;
    assert!(write_result.is_ok());

    let frame = receive_frame(&mut stream, 1024).await;
    assert!(frame.is_ok());
    assert_eq!(frame.unwrap(), TcpFrame::Pong);
}

#[tokio::test]
async fn should_answer_client_connected() {
    let server = create_server().await;
    let result = TcpStream::connect(server).await;
    assert!(result.is_ok());

    let mut stream = result.unwrap();
    let ping_frame = TcpFrame::ClientConnected;
    let mut ping_buffer = ping_frame.to_buffer();

    let write_result = stream.write_buf(&mut ping_buffer).await;
    assert!(write_result.is_ok());

    let frame = receive_frame(&mut stream, 1024).await;
    assert!(frame.is_ok());
    assert!(matches!(frame.unwrap(), TcpFrame::ClientConnectedAck { .. }));
}

#[tokio::test]
async fn should_listen_in_ack_port() {
    let server = create_server().await;
    let result = TcpStream::connect(server).await;
    assert!(result.is_ok());

    let mut stream = result.unwrap();
    let ping_frame = TcpFrame::ClientConnected;
    let mut ping_buffer = ping_frame.to_buffer();

    let _ = stream.write_buf(&mut ping_buffer).await.unwrap();

    let frame = receive_frame(&mut stream, 1024).await.unwrap();
    assert!(matches!(frame, TcpFrame::ClientConnectedAck {..}));

    let port = extract_enum_value!(frame, TcpFrame::ClientConnectedAck { port } => port);
    let ip = IpAddr::from_str("127.0.0.1").unwrap();
    let target_ip = SocketAddr::new(ip,port);

    let remote_stream = TcpStream::connect(target_ip).await;
    assert!(remote_stream.is_ok());

    let frame = receive_frame(&mut stream, 1024).await;
    assert!(frame.is_ok());
    assert!(matches!(frame.unwrap(), TcpFrame::IncomingSocket {..}));
}

#[tokio::test]
async fn should_forward_data_successfully() {
    let server = create_server().await;
    let result = TcpStream::connect(server).await;
    assert!(result.is_ok());

    let mut stream = result.unwrap();
    let ping_frame = TcpFrame::ClientConnected;
    let mut ping_buffer = ping_frame.to_buffer();

    let _ = stream.write_buf(&mut ping_buffer).await.unwrap();

    let frame = receive_frame(&mut stream, 1024).await.unwrap();
    assert!(matches!(frame, TcpFrame::ClientConnectedAck {..}));

    let port = extract_enum_value!(frame, TcpFrame::ClientConnectedAck { port } => port);
    let ip = IpAddr::from_str("127.0.0.1").unwrap();
    let target_ip = SocketAddr::new(ip,port);

    let remote_stream = TcpStream::connect(target_ip).await;
    assert!(remote_stream.is_ok());

    let frame = receive_frame(&mut stream, 1024).await;
    assert!(frame.is_ok());
    assert!(matches!(frame.unwrap(), TcpFrame::IncomingSocket {..}));

    let mut remote_stream = remote_stream.unwrap();
    let mut buffer = generate_random_buffer(1024 * 2);

    let _ = remote_stream.write_buf(&mut buffer).await.unwrap();

    let frame = receive_frame(&mut stream, 1024 * 8).await.unwrap();
    let (buffer, _) = extract_enum_value!(frame.unwrap(), TcpFrame::DataPacketHost { } => );
}

fn generate_random_buffer(buffer_size: i32) -> BytesMut {
    let mut rand = rand::thread_rng();
    let mut buffer = BytesMut::with_capacity(buffer_size as usize);

    rand.fill_bytes(&mut buffer);
    buffer
}

async fn receive_frame(stream: &mut TcpStream, buffer_size: usize) -> Result<TcpFrame, Box<dyn std::error::Error>> {
    let mut buffer = BytesMut::with_capacity(buffer_size);
    loop {
        let bytes_read = match stream.read_buf(&mut buffer).await {
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
    let listener = DefaultListener::bind(args.get_socket_addr()).await.unwrap();
    let listen_ip = listener.listen_ip();

    let mut server = Server::new(args, Box::new(listener));


    tokio::spawn(async move {
        let result = server.run(tokio::signal::ctrl_c()).await;
        println!("server sopped with result. {:?}", result);
    });

    listen_ip
}