use std::error::Error;
use mongodb::bson::{Bson, Uuid};
use tracing::error;

pub trait IntoUuid {
    fn into_uuid(&self) -> Result<Uuid, Box<dyn Error>>;
}

impl IntoUuid for Bson {
    fn into_uuid(&self) -> Result<Uuid, Box<dyn Error>> {
        if let Bson::String(value) = self {
            return Ok(Uuid::parse_str(value)?);
        }

        if let Bson::Binary(data) = self {
            return Ok(data.to_uuid()?);
        }

        error!("error trying to convert to uuid: {}", self);
        Err("failed trying to convert to uuid".into())
    }
}