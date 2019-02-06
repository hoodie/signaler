use log::*;
use std::collections::HashMap;
use crate::server::SessionId;

pub mod user;
pub use self::user::*;

mod actor;
pub use self::actor::AuthenticationRequest;

pub type UserId = String;


pub trait UserManaging {
    type UserProfile;
    fn associate_user(&mut self, name: &str, id: &SessionId, password: &str) -> Option<Self::UserProfile>;
}


pub struct UserManagement<T> {
    inner: Box<dyn UserManaging<UserProfile = T>>,
}

impl<T> UserManagement<T> {
    pub fn new(implementation: Box<dyn UserManaging<UserProfile=T>>) -> Self {
        Self {inner: implementation}
    }
}

impl<T> UserManaging for UserManagement<T> {
    type UserProfile = T;
    fn associate_user(&mut self, name: &str, id: &SessionId, password: &str) -> Option<T> {
        self.inner.associate_user(name, id, password)
    }
}

impl Default for UserManagement<UserProfile> {
    fn default() -> Self {
        NaiveUserManager::new()
    }
}


#[derive(Default)]
pub struct NaiveUserManager {
    credentials: HashMap<UserId, String>,
}

impl NaiveUserManager {
    pub fn new() -> UserManagement<UserProfile> {
       UserManagement::new(Box::new(Self::default()))
    }
}

impl UserManaging for NaiveUserManager {
    type UserProfile = UserProfile;

    fn associate_user(&mut self, name: &str, session_id: &SessionId, password: &str) -> Option<UserProfile> {
        None
    }

}
