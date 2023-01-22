use actix_web::{FromRequest, get, HttpRequest, HttpResponse, post, Responder};
use actix_web::dev::Payload;
use actix_web::web::{Data, Json, Path};
use mongodb::bson::Uuid;

use crate::AppError;
use crate::app::core::command::CommandHandler;
use crate::app::users::commands::create::CreateUserCommandHandler;
use crate::app::users::queries::{GetUserRequest, GetUserRequestHandler};
use crate::app::users::requests::{CreateUserRequest, UserDto};
use crate::app::web::parse_uuid_from_path;


#[post("")]
pub async fn create_user(cmd_handler: Data<CreateUserCommandHandler>, request: Json<CreateUserRequest>) -> Result<impl Responder, AppError> {
    let command = request.into_inner();
    let created_user = cmd_handler.execute_cmd(command).await?;
    let response = UserDto::from(created_user);

    Ok(HttpResponse::Created().json(response))
}

#[get("/{user_id}")]
pub async fn get_user(path: Path<(String, )>, cmd_handler: Data<GetUserRequestHandler>) -> Result<impl Responder, AppError> {
    let (user_id, ) = path.into_inner();
    let user_id = parse_uuid_from_path(&user_id)?;
    let command = GetUserRequest::new(&user_id);

    let user = cmd_handler.execute_cmd(command).await?;
    Ok(HttpResponse::Ok().json(UserDto::from(user)))
}