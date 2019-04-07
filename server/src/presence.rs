//! Presence Module
//!
//! PresenceService Actor that keeps track of which users are logged in and have which status at a time.
//! Work in progress...
//!

use actix::prelude::*;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

use super::*;

use std::time::{Duration, Instant};
use std::collections::HashMap;

mod simple;

pub use simple::SimplePresenceService;

/// Simple Authentication Credentials
#[derive(Debug, Serialize, Deserialize)]
pub struct UsernamePassword {
    pub username: String,
    pub password: String,
}

/// Token returned after successful authentication
///
/// Use this to make requests that require authentication. Should timeout.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct AuthToken(Uuid);
impl AuthToken {
    pub fn new() -> Self {
        Default::default()
    }
}
impl Default for AuthToken {
    fn default() -> Self {
        Self (Uuid::new_v4())
    }
}

/// General Behaviour of a PresenceService
pub trait PresenceHandler {
    type Credentials;
    type AuthToken;

    fn associate_user(&mut self, credentials: &Self::Credentials, session_id: &SessionId) -> Option<Self::AuthToken>;
    fn still_valid(&self, token: &AuthToken) -> bool;
    fn refresh(&mut self, token: &AuthToken) -> Option<AuthToken>;
    fn logout(&mut self, token: &AuthToken) -> bool;
    fn clean_up(&mut self);

}

/// Actor Container for Generic PresenceService implementations
pub struct PresenceService<C, T> {
    inner: Box<dyn PresenceHandler<Credentials=C, AuthToken=T>>
}

impl<C, T> PresenceHandler for PresenceService<C, T> {
    type Credentials = C;
    type AuthToken = T;

    fn associate_user(&mut self, credentials: &Self::Credentials, session_id: &SessionId) -> Option<Self::AuthToken> {
        self.inner.associate_user(credentials, session_id)
    }

    fn still_valid(&self, token: &AuthToken) -> bool {
        self.inner.still_valid(token)
    }

    fn refresh(&mut self, token: &AuthToken) -> Option<AuthToken> {
        self.inner.refresh(token)
    }

    fn logout(&mut self, token: &AuthToken) -> bool {
        self.inner.logout(token)
    }

    fn clean_up(&mut self) {
        self.inner.clean_up()
    }
}

impl PresenceService<UsernamePassword, AuthToken> {
    pub fn simple() -> simple::SimplePresenceService {
        Self {
            inner: Box::new(simple::SimplePresenceHandler::new())
        }
    }
}

/// Message expected by PresenceService to add SessionId
#[derive(Message)]
#[rtype(result = "Option<AuthToken>")]
pub struct AuthenticationRequest<CREDENTIALS> {
    pub credentials: CREDENTIALS,
    pub session_id: SessionId,
}

/// Message expected by PresenceService to add SessionId
#[derive(Message, Debug)]
#[rtype(result = "bool")]
pub struct ValidateRequest {
    pub token: AuthToken,
}