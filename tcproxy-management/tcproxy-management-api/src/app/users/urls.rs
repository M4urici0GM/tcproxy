use actix_web::{get, HttpResponse, post, Responder};
use actix_web::web::{Data, Json, Path};
use bcrypt::{DEFAULT_COST, hash};
use mongodb::bson::{doc, Uuid};
use crate::app::users::{CreateUserRequest, User, UserRepository};
use crate::{AppError, AppErrorType, Validator};

fn parse_uuid_from_path(uuid: &str) -> Result<Uuid, AppError> {
    match Uuid::parse_str(uuid) {
        Ok(uuid) => Ok(uuid),
        Err(_) => {
            let msg = format!("invalid uuid: {}", uuid);
            Err(AppError::new(&msg, AppErrorType::BadRequestError, None))
        }
    }
}

#[post("")]
pub async fn create_user(user_repo: Data<UserRepository>, request: Json<CreateUserRequest>) -> Result<impl Responder, AppError> {
    request.validate()?;

    let user_exists = user_repo
        .exist_by_username_or_email(request.username(), request.email())
        .await?;

    if user_exists {
        let msg = format!("an user with username {} or email {} already exists", request.username(), request.email());
        return Err(AppError::new(&msg, AppErrorType::ConflictError, None));
    }

    let password_hash = hash(request.password(), DEFAULT_COST).unwrap();
    let user = User::new(
        None,
        request.username(),
        request.name(),
        &password_hash,
        request.email());

    let user_id =  user_repo.new_user(user).await?;

    Ok(HttpResponse::Created().json(doc! { "user_id": user_id.to_string() }))
}

#[get("/{user_id}")]
pub async fn get_user(path: Path<(String,)>, user_repo: Data<UserRepository>) -> Result<impl Responder, AppError> {
    let (user_id,) = path.into_inner();
    let user_id = parse_uuid_from_path(&user_id)?;
    let user = user_repo.get_user(user_id).await?;

    Ok(HttpResponse::Ok().json(user))
}