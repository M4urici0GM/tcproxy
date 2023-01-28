use mongodb::bson::Uuid;
use crate::{HttpAppError, StatusCode};

pub fn parse_uuid_from_path(uuid: &str) -> Result<Uuid, HttpAppError> {
    match Uuid::parse_str(uuid) {
        Ok(uuid) => Ok(uuid),
        Err(_) => {
            let msg = format!("invalid uuid: {}", uuid);
            Err(HttpAppError::new(&msg, StatusCode::BadRequestError, None))
        }
    }
}
