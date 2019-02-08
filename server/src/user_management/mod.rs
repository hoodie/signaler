use log::*;
use serde_derive::{Deserialize, Serialize};

use std::collections::HashMap;

pub mod user;
pub use self::user::*;

mod naive;
pub use self::naive::AuthenticationRequest;

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