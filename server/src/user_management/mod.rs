//! Communicates profice and Authentication Data

use actix::prelude::*;
#[allow(unused_imports)]
use log::{debug, error, info, trace};

use std::time::Duration;

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
    fn update(&mut self);
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
    fn update(&mut self) {
        trace!("UserManager updating");
        self.inner.update()
    }
}

impl Actor for UserService {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Context<Self>) {
        ctx.run_interval(Duration::from_millis(5_000), |slf, _| slf.update());
        trace!("usermanager started");
    }
}

impl SystemService for UserService {}
impl Supervised for UserService {}

impl Default for UserService {
    fn default() -> Self {
        NaiveUserManager::naive()
    }
}
