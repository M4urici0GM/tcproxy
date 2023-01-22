pub mod requests;
pub mod urls;
pub mod data;
pub mod model;
pub mod commands;
pub mod queries;

use actix_web::web::{Data, ServiceConfig};
use mongodb::Database;
use crate::app::users::commands::create::CreateUserCommandHandler;

use crate::app::users::data::{UserRepositoryReaderImpl, UserRepositoryWriterImpl};
use crate::app::users::queries::GetUserRequestHandler;
use crate::app::users::urls::{create_user, get_user};

pub fn register_user_urls(config: &mut ServiceConfig) {
    let user_scopes = actix_web::web::scope("/users")
        .service(get_user)
        .service(create_user);

    config.service(user_scopes);
}

pub fn register_user_services(config: &mut ServiceConfig, database: &Database) {
    let repository_writer = UserRepositoryWriterImpl::new(database);
    let repository_reader = UserRepositoryReaderImpl::new(database);

    let get_user_handler = GetUserRequestHandler::new(&repository_reader);
    let create_user_handler = CreateUserCommandHandler::new(&repository_reader, &repository_writer);

    config.app_data(Data::new(create_user_handler));
    config.app_data(Data::new(get_user_handler));
}