
use std::env;
use std::fmt::{Debug, Display, Formatter};
use std::future::ready;
use std::task::{Context, Poll};
use actix_web::{App, get, HttpResponse, HttpServer, post, Responder, ResponseError};
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::web::{BytesMut, Data, Json, Path};
use dotenv::dotenv;
use mongodb::bson::oid::ObjectId;
use mongodb::{Client, Collection, Database};
use mongodb::bson::{Bson, doc, Uuid};
use serde::{Deserialize, Serialize};
use tcproxy_core::Error;
use actix_web::http::StatusCode;
use serde::de::StdError;
use std::str::FromStr;
use actix_web::body::BoxBody;
use actix_web::middleware::{Compat, Logger};
use actix_web_opentelemetry::RequestTracing;
use tracing_actix_web::TracingLogger;
use opentelemetry::sdk::export::trace::stdout;
use opentelemetry::sdk::propagation::TraceContextPropagator;
use opentelemetry::sdk::{trace, Resource};
use opentelemetry::KeyValue;
use opentelemetry::{global};
use std::collections::HashMap;
use tracing::{error, info, Subscriber};
use tracing::log::debug;
use tracing::subscriber::set_global_default;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tcproxy_management_api::app::register_services;


// struct ErrorMiddlewareTransform;
//
// struct ErrorMiddleware<S> {
//     service: S
// }



// impl<S, B> Transform<S, ServiceRequest> for ErrorMiddlewareTransform
// where
//     S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error>,
//     S::Future: 'static,
//     B: 'static {
//     type Response = ServiceResponse<B>;
//     type Error = actix_web::Error;
//     type Transform = ErrorMiddleware<S>;
//     type InitError = ();
//     type Future = std::future::Ready<Result<Self::Transform, Self::InitError>>;
//
//     fn new_transform(&self, service: S) -> Self::Future {
//         ready(Ok(ErrorMiddleware { service }))
//     }
// }
//
// impl<S, B> Service<ServiceRequest> for ErrorMiddleware<S>
// where
//     S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error>,
//     S::Future: 'static,
//     B: 'static
// {
//     type Response = ServiceResponse<B>;
//     type Error = actix_web::Error;
//     type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;
//
//     forward_ready!(service);
//
//     fn call(&self, req: ServiceRequest) -> Self::Future {
//         println!("Hi from start. You requested: {}", req.path());
//
//         let fut = self.service.call(req);
//
//         Box::pin(async move {
//             let res: ServiceResponse<B> = fut.await?;
//
//             println!("Hi from response");
//             Ok(res)
//         })
//     }
// }

#[actix_web::main]
async fn main() -> std::io::Result<()>{
    dotenv().ok();
    tcproxy_management_api::fmt::init_subscriber();

    let uri = match env::var("MONGOURI") {
        Ok(v) => v.to_string(),
        Err(_) => format!("Error loading env variable"),
    };

    let client = Client::with_uri_str(uri).await.unwrap();
    let database = client.database("tcproxy");

    HttpServer::new(move || {
        App::new()
            // .wrap(ErrorMiddlewareTransform {})
            .wrap(Logger::default())
            .configure(tcproxy_management_api::app::register_urls)
            .configure(|config| { register_services(config, &database) })
    })
    .bind(("127.0.0.1", 3333))?
    .run()
    .await
}
