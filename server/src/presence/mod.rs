//! Presence Module
//!
//! PresenceService Actor that keeps track of which users are logged in and have which status at a time.
//! Work in progress...
//!

use actix::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::*;
use crate::user_management::UserProfile;

use std::collections::HashMap;
use std::time::{Duration, Instant};

pub mod command;
pub mod message;
mod session_manager;
mod simple;

pub use signaler_protocol::Credentials;

use command::AuthToken;

/// General Behavior of a PresenceService
pub trait PresenceHandler {
    type Credentials;
    type AuthToken;

    fn associate_user(
        &mut self,
        credentials: &Self::Credentials,
        session_id: &SessionId,
    ) -> Option<message::AuthResponse<Self::AuthToken, UserProfile>>;
    fn still_valid(&self, token: &AuthToken) -> bool;
    fn refresh_token(&mut self, token: &AuthToken) -> Option<AuthToken>;
    fn logout(&mut self, token: &AuthToken) -> bool;
    fn reload_users(&mut self);
    fn clean_up(&mut self);
}

/// Actor Container for Generic PresenceService implementations
pub struct PresenceService<C, T> {
    inner: Box<dyn PresenceHandler<Credentials = C, AuthToken = T>>,
}

/// Simple Presence Service
pub type SimplePresenceService = PresenceService<Credentials, AuthToken>;

impl<C, T> PresenceHandler for PresenceService<C, T> {
    type Credentials = C;
    type AuthToken = T;

    fn associate_user(
        &mut self,
        credentials: &Self::Credentials,
        session_id: &SessionId,
    ) -> Option<message::AuthResponse<Self::AuthToken, UserProfile>> {
        self.inner.associate_user(credentials, session_id)
    }

    fn still_valid(&self, token: &AuthToken) -> bool {
        self.inner.still_valid(token)
    }

    fn refresh_token(&mut self, token: &AuthToken) -> Option<AuthToken> {
        self.inner.refresh_token(token)
    }

    fn reload_users(&mut self) {
        self.inner.reload_users()
    }

    fn logout(&mut self, token: &AuthToken) -> bool {
        self.inner.logout(token)
    }

    fn clean_up(&mut self) {
        self.inner.clean_up()
    }
}

impl Actor for PresenceService<Credentials, AuthToken> {
    type Context = Context<Self>;
    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.run_interval(Duration::from_millis(30_000), |slf, _| slf.reload_users());
        debug!("presence started");
    }
}

impl Default for PresenceService<Credentials, AuthToken> {
    fn default() -> Self {
        Self {
            inner: Box::new(simple::SimplePresenceHandler::new()),
        }
    }
}

impl SystemService for PresenceService<Credentials, AuthToken> {}
impl Supervised for PresenceService<Credentials, AuthToken> {}
