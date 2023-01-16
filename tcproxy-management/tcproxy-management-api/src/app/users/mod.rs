mod requests;
mod services;
mod urls;
mod data;
mod model;

use actix_web::web::{Data, ServiceConfig};
use mongodb::Database;
pub use requests::*;
pub use services::*;
pub use urls::*;
pub use data::*;
pub use model::*;

pub fn register_user_urls(config: &mut ServiceConfig) {
    let user_scopes = actix_web::web::scope("/users")
        .service(get_user)
        .service(create_user);

    config.service(user_scopes);
}

pub fn register_user_services(config: &mut ServiceConfig, database: &Database) {
    config.app_data(Data::new(UserRepository::new(&database)));
}