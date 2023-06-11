use uuid::Uuid;

pub mod token_handler;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct User {
    id: Uuid,
    name: String,
    email: String,
    password_hash: String,
}

impl User {
    pub fn id(&self) -> &Uuid {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }
    
    pub fn email(&self) -> &str {
        &self.email
    }
    
    pub fn password(&self) -> &str {
        &self.password_hash
    }
}

impl User {
    pub fn new(id: &Uuid, name: &str, email: &str, password: &str) -> Self {
        Self {
            id: id.clone(),
            name: String::from(name),
            email: String::from(email),
            password_hash: String::from(password),
        }
    }
}
