use actix_web::{HttpResponse, post, Responder};
use actix_web::web::{Data, Json};


use crate::app::auth::commands::AuthenticateRequestHandler;
use crate::app::auth::requests::AuthenticateRequest;
use crate::app::core::command::CommandHandler;
use crate::HttpAppError;


#[post("")]
pub async fn authenticate(request: Json<AuthenticateRequest>, handler: Data<AuthenticateRequestHandler>) -> Result<impl Responder, HttpAppError> {
    let command = request.into_inner();
    let result = handler.execute_cmd(command).await?;
    Ok(HttpResponse::Created()
        .json(result))
}