use bcrypt::DEFAULT_COST;
use diesel::result::Error::NotFound;
use diesel::{insert_into, prelude::*};
use tcproxy_core::auth::User;
use tracing::{error, info};
use uuid::Uuid;

use crate::models::{self, UserModel};
use crate::schema::{self, users};

#[derive(Debug)]
pub enum AccountManagerError {
    NotFound,
    Other(tcproxy_core::Error),
}

impl TryFrom<UserModel> for User {
    type Error = tcproxy_core::Error;

    fn try_from(value: UserModel) -> Result<Self, Self::Error> {
        let user_id = Uuid::from_slice(value.id())?;

        Ok(Self::new(
            &user_id,
            value.name(),
            value.email(),
            value.password(),
        ))
    }
}

pub trait UserManager: Send + Sync {
    fn find_account_by_id(&self, account_id: &Uuid) -> Result<User, AccountManagerError>;
    fn find_user_by_email(&self, email: &str) -> Result<User, AccountManagerError>;
}

pub struct DefaultAccountManager {}

impl DefaultAccountManager {
    pub fn new() -> Self {
        Self {}
    }
}

impl DefaultAccountManager {
    pub fn create_default_user(&self) -> tcproxy_core::Result<()> {
        use schema::users::dsl::*;

        match self.find_user_by_email("admin@admin.org") {
            Ok(_) => {
                info!("default user already exists... skipping");
                return Ok(());
            }
            Err(AccountManagerError::NotFound) => {
                info!("default user doesnt exist, creating one..");
            }
            Err(AccountManagerError::Other(err)) => {
                error!(
                    "error when trying to checking if default user exists.. {:?}",
                    err
                );
                return Err(err);
            }
        };

        let connection = &mut SqliteConnection::establish("file:tcproxy.db").unwrap();
        let user_id = Uuid::new_v4();
        let pass_hash = bcrypt::hash("1234", DEFAULT_COST).unwrap(); // TODO: fixme
        let default_user = UserModel::new(&user_id, "Default User", "admin@admin.org", &pass_hash);

        match insert_into(users).values(&default_user).execute(connection) {
            Ok(_) => {
                info!("created default user {}", default_user.email());
                Ok(())
            }
            Err(err) => {
                error!("error when trying to create default user: {}", err);
                Err(err.into())
            }
        }
    }
}

impl UserManager for DefaultAccountManager {
    fn find_account_by_id(&self, account_id: &Uuid) -> Result<User, AccountManagerError> {
        let connection = &mut SqliteConnection::establish("file:tcproxy.db").unwrap();
        let id_bytes = account_id.as_bytes().to_vec();
        let maybe_user = users::dsl::users.find(id_bytes).first(connection); // TODO: fixme

        let user_details: UserModel = match maybe_user {
            Ok(user) => user,
            Err(NotFound) => {
                error!("unable to find user with id: {}", account_id);
                return Err(AccountManagerError::NotFound);
            }
            Err(err) => {
                error!("Failed when trying to find user: {}", err);
                return Err(AccountManagerError::Other(err.into()));
            }
        };

        Ok(User::try_from(user_details).unwrap()) // TODO: Fixme
    }

    fn find_user_by_email(&self, email: &str) -> Result<User, AccountManagerError> {
        let connection = &mut SqliteConnection::establish("file:tcproxy.db").unwrap();
        let maybe_user: Result<Vec<UserModel>, diesel::result::Error> = users::dsl::users
            .filter(schema::users::email.eq(email))
            .limit(1)
            .select(models::UserModel::as_select())
            .load(connection);

        let user_details = match maybe_user {
            Ok(user) => {
                if user.len() == 0 {
                    return Err(AccountManagerError::NotFound);
                }

                user.get(0).ok_or(AccountManagerError::NotFound)?.to_owned()
            }
            Err(err) => {
                error!("Failed when trying to find user: {}", err);
                return Err(AccountManagerError::Other(err.into()));
            }
        };

        Ok(User::try_from(user_details).unwrap())
    }
}
