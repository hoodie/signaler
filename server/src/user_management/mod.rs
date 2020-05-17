//! Communicates profice and Authentication Data

use actix::prelude::*;
#[allow(unused_imports)]
use log::{debug, error, info};

pub mod command;
pub mod message;
mod naive;

pub use message::*;
use naive::*;

pub type UserId = String;
type UserService = UserManager<UserProfile>;

pub trait UserManaging {
    type UserProfile;
    fn who_is(&self, user_id: &str) -> Option<Self::UserProfile>;
}

pub struct UserManager<P> {
    inner: Box<dyn UserManaging<UserProfile = P>>,
}

impl<P> UserManager<P> {
    pub fn new(implementation: Box<dyn UserManaging<UserProfile = P>>) -> Self {
        Self {
            inner: implementation,
        }
    }
}

impl<P> UserManaging for UserManager<P> {
    type UserProfile = P;
    fn who_is(&self, user_id: &str) -> Option<Self::UserProfile> {
        self.inner.who_is(user_id)
    }
}

impl Actor for UserManager<UserProfile> {
    type Context = Context<Self>;
}

impl SystemService for UserService {}
impl Supervised for UserService {}

impl Default for UserService {
    fn default() -> Self {
        NaiveUserManager::naive()
    }
}

impl UserManaging for NaiveUserManager {
    type UserProfile = UserProfile;

    fn who_is(&self, user_id: &str) -> Option<UserProfile> {
        if let Some(profile) = self.user_database.profiles.get(user_id) {
            info!("found profile {:?}", user_id);
            Some(profile.clone())
        } else {
            error!("found user but not profile {:?}", user_id);
            None
        }
    }
}
