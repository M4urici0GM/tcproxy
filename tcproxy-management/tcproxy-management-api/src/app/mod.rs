mod users;

pub mod bson;
pub mod data;
pub mod core;
pub mod web;
pub mod auth;

use actix_web::web::ServiceConfig;
use mongodb::Database;

pub fn register_urls(config: &mut ServiceConfig) {
    users::register_user_urls(config);
    auth::register_auth_urls(config);
}

pub fn register_services(config: &mut ServiceConfig, database: &Database) {
    users::register_user_services(config, database);
    auth::register_auth_services(config, database);
}
