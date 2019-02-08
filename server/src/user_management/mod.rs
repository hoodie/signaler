use log::*;
use serde_derive::{Deserialize, Serialize};

use std::collections::HashMap;

use crate::server::SessionId;

pub mod user;
pub use self::user::*;

mod actor;
pub use self::actor::AuthenticationRequest;

pub type UserId = String;


pub trait UserManaging {
    type UserProfile;
    type Credentials;
    fn associate_user(&mut self, credentials: &Self::Credentials) -> Option<Self::UserProfile>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UsernamePassword {
    pub username: String,
    pub password: String,
}

pub struct UserManagement<P, C> {
    inner: Box<dyn UserManaging<UserProfile = P, Credentials = C>>,
}

impl<P, C> UserManagement<P, C> {
    pub fn new(implementation: Box<dyn UserManaging<UserProfile=P, Credentials=C>>) -> Self {
        Self { inner: implementation }
    }
}

impl<P, C> UserManaging for UserManagement<P, C> {
    type UserProfile = P;
    type Credentials = C;
    fn associate_user(&mut self, credentials: &Self::Credentials) -> Option<Self::UserProfile> {
        self.inner.associate_user(credentials)
    }
}

impl Default for UserManagement<UserProfile, UsernamePassword> {
    fn default() -> Self {
        NaiveUserManager::new()
    }
}


#[derive(Debug, Default)]
pub struct NaiveUserManager {
    credentials: HashMap<UserId, String>,
}

impl NaiveUserManager {
    pub fn new() -> UserManagement<UserProfile, UsernamePassword> {
        let manager = NaiveUserManager {
            credentials: serde_json::from_str(include_str!("../../testcredentials.json")).unwrap()
        };
        debug!("new NaiveUserManager {:?}", manager);
        UserManagement::new(Box::new(manager))
    }
}

impl UserManaging for NaiveUserManager {
    type UserProfile = UserProfile;
    type Credentials = UsernamePassword;

    fn associate_user(&mut self, _credentials: &Self::Credentials) -> Option<UserProfile> {
        None
    }

}
