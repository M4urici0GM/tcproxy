use mongodb::bson::Uuid;
use crate::{AppError, AppErrorType};

pub fn parse_uuid_from_path(uuid: &str) -> Result<Uuid, AppError> {
    match Uuid::parse_str(uuid) {
        Ok(uuid) => Ok(uuid),
        Err(_) => {
            let msg = format!("invalid uuid: {}", uuid);
            Err(AppError::new(&msg, AppErrorType::BadRequestError, None))
        }
    }
}
