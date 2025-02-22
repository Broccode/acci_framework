use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    pub last_login: Option<OffsetDateTime>,
    pub is_active: bool,
    pub is_verified: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUser {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginCredentials {
    pub email: String,
    pub password: String,
}

#[derive(Debug, thiserror::Error)]
pub enum UserError {
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("User not found")]
    NotFound,
    #[error("User already exists")]
    AlreadyExists,
    #[error("Password too weak: {0}")]
    WeakPassword(String),
    #[error("Invalid email format")]
    InvalidEmail,
    #[error("User is not active")]
    InactiveUser,
    #[error("User is not verified")]
    UnverifiedUser,
    #[error("Database error: {0}")]
    DatabaseError(String),
}

impl User {
    pub fn new(email: String, password_hash: String) -> Self {
        let now = OffsetDateTime::now_utc();
        Self {
            id: Uuid::new_v4(),
            email,
            password_hash,
            created_at: now,
            updated_at: now,
            last_login: None,
            is_active: true,
            is_verified: false,
        }
    }

    pub fn update_last_login(&mut self) {
        self.last_login = Some(OffsetDateTime::now_utc());
        self.updated_at = OffsetDateTime::now_utc();
    }
}

#[async_trait]
pub trait UserRepository: Send + Sync + 'static {
    async fn create(&self, user: &User) -> Result<(), UserError>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, UserError>;
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, UserError>;
    async fn update(&self, user: &User) -> Result<(), UserError>;
    async fn delete(&self, id: Uuid) -> Result<(), UserError>;
    async fn verify_email(&self, id: Uuid) -> Result<(), UserError>;
    async fn deactivate(&self, id: Uuid) -> Result<(), UserError>;
    async fn activate(&self, id: Uuid) -> Result<(), UserError>;
}

// Mock-Implementation für Tests
#[cfg(test)]
pub mod mock {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Mutex;

    pub struct MockUserRepository {
        users: Mutex<HashMap<Uuid, User>>,
    }

    impl MockUserRepository {
        pub fn new() -> Self {
            Self {
                users: Mutex::new(HashMap::new()),
            }
        }
    }

    #[async_trait]
    impl UserRepository for MockUserRepository {
        async fn create(&self, user: &User) -> Result<(), UserError> {
            let mut users = self.users.lock().unwrap();
            if users.values().any(|u| u.email == user.email) {
                return Err(UserError::AlreadyExists);
            }
            users.insert(user.id, user.clone());
            Ok(())
        }

        async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, UserError> {
            let users = self.users.lock().unwrap();
            Ok(users.get(&id).cloned())
        }

        async fn find_by_email(&self, email: &str) -> Result<Option<User>, UserError> {
            let users = self.users.lock().unwrap();
            Ok(users.values().find(|u| u.email == email).cloned())
        }

        async fn update(&self, user: &User) -> Result<(), UserError> {
            let mut users = self.users.lock().unwrap();
            if users.contains_key(&user.id) {
                users.insert(user.id, user.clone());
                Ok(())
            } else {
                Err(UserError::NotFound)
            }
        }

        async fn delete(&self, id: Uuid) -> Result<(), UserError> {
            let mut users = self.users.lock().unwrap();
            if users.remove(&id).is_some() {
                Ok(())
            } else {
                Err(UserError::NotFound)
            }
        }

        async fn verify_email(&self, id: Uuid) -> Result<(), UserError> {
            let mut users = self.users.lock().unwrap();
            if let Some(user) = users.get_mut(&id) {
                user.is_verified = true;
                user.updated_at = OffsetDateTime::now_utc();
                Ok(())
            } else {
                Err(UserError::NotFound)
            }
        }

        async fn deactivate(&self, id: Uuid) -> Result<(), UserError> {
            let mut users = self.users.lock().unwrap();
            if let Some(user) = users.get_mut(&id) {
                user.is_active = false;
                user.updated_at = OffsetDateTime::now_utc();
                Ok(())
            } else {
                Err(UserError::NotFound)
            }
        }

        async fn activate(&self, id: Uuid) -> Result<(), UserError> {
            let mut users = self.users.lock().unwrap();
            if let Some(user) = users.get_mut(&id) {
                user.is_active = true;
                user.updated_at = OffsetDateTime::now_utc();
                Ok(())
            } else {
                Err(UserError::NotFound)
            }
        }
    }
}
