pub mod error;

use std::error::Error;
use std::fmt::{Display, Formatter};
use async_trait::async_trait;
use mongodb::bson::{Document, Uuid};

#[derive(Debug)]
pub enum RepositoryError {
    NotFound { message: String },
    Other(Box<dyn Error>)
}

pub type RepositoryResult<T> = Result<T, RepositoryError>;

#[async_trait]
pub trait RepositoryWriter<T>: Sync + Send {
    async fn insert_one(&self, entity: &T) -> RepositoryResult<Uuid>;
}

#[async_trait]
pub trait RepositoryReader<T>: Sync + Send {
    async fn find_by_id(&self, entity_id: Uuid) -> RepositoryResult<Option<T>>;
    async fn find_one(&self, query: Document) -> RepositoryResult<Option<T>>;
}

impl From<mongodb::error::Error> for RepositoryError {
    fn from(value: mongodb::error::Error) -> Self {
        Self::Other(value.into())
    }
}

impl Display for RepositoryError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
