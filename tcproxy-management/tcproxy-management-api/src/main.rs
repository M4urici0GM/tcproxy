
use std::env;



use actix_web::{App, HttpServer};


use dotenv::dotenv;

use mongodb::{Client};







use actix_web::middleware::{Logger};













use tcproxy_management_api::app::{auth, register_services};


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

    // TODO: Implement a HttpServerConfig or something
    // for centralizing config.
    let uri = match env::var("MONGOURI") {
        Ok(v) => v,
        Err(_) => "Error loading env variable".to_string(),
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
