use tracing::{info};
use clap::Parser;
use tokio::signal;

use tcproxy::{Result, AppArguments, Server};

// enum Message {
//     Connected,
//     DataBytes(BytesMut),
//     Disconnected,
// }

// async fn handle_socket(tcp_stream: &mut TcpStream) -> Result<()> {
//     let mut target_stream = match TcpStream::connect("45.77.198.191:19132").await {
//         Ok(stream) => stream,
//         Err(err) => {
//             error!("Failed when trying to connect to destination {}", err);
//             return Ok(());
//         }
//     };
//
//     let mut stream_duplex = DuplexTcpStream::join(tcp_stream, &mut target_stream, None);
//     match stream_duplex.start().await {
//         Ok(_) => info!("Successfully streamed data."),
//         Err(_) => {}
//     };
//
//     Ok(())
// }


#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let args = AppArguments::parse();
    let shutdown_signal = signal::ctrl_c();
    Server::new(args)
        .run(shutdown_signal)
        .await?;

    info!("server stopped");
    Ok(())
}
