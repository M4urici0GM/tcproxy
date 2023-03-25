use std::fmt::{Display, Formatter};
use bytes::Bytes;
use http_body_util::{BodyExt, Empty};
use hyper::body::{Body, Buf, Incoming};
use hyper::{Method, Request, Response, Uri};
use hyper::http::uri::Scheme;
use serde::de::{DeserializeOwned, StdError};
use tokio::net::TcpStream;
use tokio_native_tls::native_tls::TlsConnector;


mod api_client;
pub use api_client::*;

// async fn send_https_request<B>(request: Request<B>) -> Result<Response<Incoming>, Box<dyn std::error::Error + 'static>>
//     where
//         B: Sync + Send,
//         B: Body + 'static,
//         B::Data: Send,
//         B::Error: Into<Box<dyn StdError + Send + Sync>>
// {
//     let hyper_uri = request.uri();
//
//     let host = hyper_uri.host().unwrap();
//     let port = hyper_uri.port_u16().unwrap_or(443);
//
//     let address = format!("{}:{}", host, port);
//     let stream = TcpStream::connect(address).await?;
//
//     let ctx = TlsConnector::builder().build()?;
//     let ctx = tokio_native_tls::TlsConnector::from(ctx);
//     let stream = ctx.connect(host, stream).await?;
//
//     let (mut sender, conn) = hyper::client::conn::http1::handshake::<_, B>(stream).await?;
//     tokio::spawn(async move {
//         if let Err(err) = conn.await {
//             println!("failed to connect to server {:?}", err);
//         }
//     });
//
//     Ok(sender.send_request(request).await?)
// }
//
// async fn send_http_request<B>(request: Request<B>) -> Result<Response<Incoming>, Box<dyn std::error::Error + 'static>>
//     where
//         B: Sync + Send,
//         B: Body + 'static,
//         B::Data: Send,
//         B::Error: Into<Box<dyn StdError + Send + Sync>>
// {
//     let hyper_uri = request.uri();
//
//     let host = hyper_uri.host().unwrap();
//     let port = hyper_uri.port_u16().unwrap_or(443);
//
//     let address = format!("{}:{}", host, port);
//     let stream = TcpStream::connect(address).await?;
//
//     let (mut sender, conn) = hyper::client::conn::http1::handshake::<_, B>(stream).await?;
//     tokio::spawn(async move {
//         if let Err(err) = conn.await {
//             println!("failed to connect to server {:?}", err);
//         }
//     });
//
//     Ok(sender.send_request(request).await?)
// }
//
// async fn send_request<R, B>(request: Request<B>) -> Result<R, Box<dyn std::error::Error + 'static>>
//     where
//         R: DeserializeOwned,
//         B: Sync + Send,
//         B: Body + 'static,
//         B::Data: Send,
//         B::Error: Into<Box<dyn StdError + Send + Sync>>
// {
//     let hyper_uri = request.uri();
//     let scheme = hyper_uri.scheme().unwrap();
//
//     let response = match scheme {
//         &Scheme::HTTPS => send_https_request(request).await?,
//         &Scheme::HTTP => send_http_request(request).await?,
//     };
//
//     let response_body = response.collect().await?.aggregate();
//     let response_body_reader = response_body.reader();
//
//     Ok(serde_json::from_reader(response_body_reader)?)
// }
//
//
//
// pub async fn get_google_test() -> Result<(), Box<dyn std::error::Error + Sync + Send + 'static>>{
//     let request = Request::builder()
//         .uri("https://www.google.com")
//         .header(hyper::header::HOST, "google.com")
//         .method(Method::GET)
//         .body(Empty::<Bytes>::new())?;
//
//     let response = match send_request::<>(request).await {
//         Ok(response) => response,
//         Err(err) => {
//             println!("{:?}", err);
//             return Err("aaaa".into());
//         }
//     };
//
//     Ok(())
// }