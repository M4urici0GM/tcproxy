use std::sync::Mutex;
use chrono::{DateTime, Utc};
use tcproxy_core::auth::AccountDetailsDto;

pub struct AuthenticationManager {
    is_authenticated: bool,
    account_details: Option<AccountDetailsDto>,
    authenticated_at: Option<DateTime<Utc>>,
}

pub struct AuthenticationManagerGuard {
    manager: Mutex<AuthenticationManager>,
}

impl AuthenticationManagerGuard {
    pub fn new(manager: AuthenticationManager) -> Self {
        Self {
            manager: Mutex::new(manager),
        }
    }
    
    pub fn is_authenticated(&self) -> bool {
        let lock = self.manager
            .lock()
            .unwrap();
        
        lock.is_authenticated()
    }

    pub fn set_authentication_details(&self, details: &AccountDetailsDto) {
        let mut lock = self.manager
            .lock()
            .unwrap();

        lock.set_authentication_details(details.clone());
    }

    pub fn revoke_authentication(&self) {
        let mut lock = self.manager
            .lock()
            .unwrap();

        lock.revoke_authentication();
    }
}

impl AuthenticationManager {
    pub fn new() -> Self {
        Self {
            is_authenticated: false,
            account_details: None,
            authenticated_at: None,
        }
    }

    pub fn is_authenticated(&self) -> bool {
        self.is_authenticated
    }

    pub fn account_details(&self) -> Option<AccountDetailsDto> {
        self.account_details.clone()
    }

    pub fn authenticated_at(&self) -> &Option<DateTime<Utc>> {
        &self.authenticated_at
    }

    pub fn set_authentication_details(&mut self, details: AccountDetailsDto) {
        self.is_authenticated = true;
        self.authenticated_at = Some(Utc::now());
        self.account_details = Some(details);
    }

    pub fn revoke_authentication(&mut self) {
        self.is_authenticated = false;
        self.authenticated_at = None;
        self.account_details = None;
    }
}

#[cfg(test)]
pub mod tests {
    use mongodb::bson::Uuid;
    use tcproxy_core::auth::{AccountDetailsDto, UserDetails};
    use crate::managers::AuthenticationManager;

    #[test]
    pub fn should_set_authentication_correctly() {
        // Arrange
        let user_details = UserDetails::new(
            &Uuid::new(),
            "some name",
            "some@email.com");

        let account_details = AccountDetailsDto::new(&Uuid::new(), "account-name", &user_details);
        let mut auth_manager = AuthenticationManager::new();

        // Act
        auth_manager.set_authentication_details(account_details.clone());

        // Assert
        assert!(auth_manager.is_authenticated);
        assert!(auth_manager.account_details.is_some());
        assert!(auth_manager.authenticated_at.is_some());

        let got_account_details = auth_manager.account_details().unwrap();

        assert_eq!(got_account_details, account_details);
    }

    #[test]
    pub fn should_revoke_authentication_correctly() {
        // Arrange
        let user_details = UserDetails::new(
            &Uuid::new(),
            "some name",
            "some@email.com");

        let account_details = AccountDetailsDto::new(&Uuid::new(), "account-name", &user_details);
        let mut auth_manager = AuthenticationManager::new();
        auth_manager.set_authentication_details(account_details.clone());

        // Act
        auth_manager.revoke_authentication();

        // Assert
        assert!(!auth_manager.is_authenticated);
        assert!(auth_manager.account_details.is_none());
        assert!(auth_manager.authenticated_at.is_none());
    }
}