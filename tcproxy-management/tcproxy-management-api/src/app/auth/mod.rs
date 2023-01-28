use actix_web::web;
use actix_web::web::{Data, ServiceConfig};
use mongodb::Database;
use crate::app::auth::commands::AuthenticateRequestHandler;
use crate::app::auth::urls::{authenticate};
use crate::app::users::data::UserRepositoryReaderImpl;

pub mod urls;
pub mod requests;
pub mod commands;


pub fn register_auth_urls(config: &mut ServiceConfig) {
    let scope = web::scope("/api/v1/auth")
        .service(authenticate);

    config.service(scope);
}

pub fn register_auth_services(config: &mut ServiceConfig, database: &Database) {
    let user_reader = UserRepositoryReaderImpl::new(database);
    let authenticate_request_handler = AuthenticateRequestHandler::new(user_reader);

    config.app_data(Data::new(authenticate_request_handler));
}