//! Presence Module
//!
//! PresenceService Actor that keeps track of which users are logged in and have which status at a time.
//! Work in progress...
//!

use actix::prelude::*;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

use crate::user_management::UserProfile;
use super::*;

use std::time::{Duration, Instant};
use std::collections::HashMap;

mod simple;

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

#[derive(Debug)]
pub struct AuthResponse<T, P> {
    pub token: T,
    pub profile: Option<P>,
}

/// General Behaviour of a PresenceService
pub trait PresenceHandler {
    type Credentials;
    type AuthToken;

    fn associate_user(&mut self, credentials: &Self::Credentials, session_id: &SessionId) -> Option<AuthResponse<Self::AuthToken, UserProfile>>;
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

    fn associate_user(&mut self, credentials: &Self::Credentials, session_id: &SessionId) -> Option<AuthResponse<Self::AuthToken, UserProfile>> {
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

pub type SimpleAuthResponse = AuthResponse<AuthToken, UserProfile>;

/// Message expected by PresenceService to add SessionId
#[derive(Message)]
#[rtype(result = "Option<SimpleAuthResponse>")]
pub struct AuthenticationRequest<CREDENTIALS> {
    pub credentials: CREDENTIALS,
    pub session_id: SessionId,
}

/// implementation docs
impl Handler<AuthenticationRequest<UsernamePassword>> for PresenceService<UsernamePassword, AuthToken> {
    type Result = MessageResult<AuthenticationRequest<UsernamePassword>>;

    fn handle(&mut self, request: AuthenticationRequest<UsernamePassword>, _ctx: &mut Self::Context) -> Self::Result {
        info!("received AuthenticationRequest");
        let AuthenticationRequest { credentials, session_id } = request;
        MessageResult(self.associate_user(&credentials, &session_id))
    }
}

/// Message expected by PresenceService to add SessionId
#[derive(Message, Debug)]
#[rtype(result = "bool")]
pub struct ValidateRequest {
    pub token: AuthToken,
}

impl Handler<ValidateRequest> for PresenceService<UsernamePassword, AuthToken> {
    type Result = MessageResult<ValidateRequest>;
    fn handle(&mut self, request: ValidateRequest, _ctx: &mut Self::Context) -> Self::Result {
        let ValidateRequest {token} = request;
        MessageResult(self.still_valid(&token))
    }
}

impl Actor for PresenceService<UsernamePassword, AuthToken> {
    type Context = Context<Self>;
    fn started(&mut self, _ctx: &mut Self::Context) {
        debug!("presence started");
    }
}

impl Default for PresenceService<UsernamePassword, AuthToken> {
    fn default() -> Self {
        Self {
            inner: Box::new(simple::SimplePresenceHandler::new())
        }
    }
}

impl SystemService for PresenceService<UsernamePassword, AuthToken> {}
impl Supervised for PresenceService<UsernamePassword, AuthToken> {}

/// Simple Presence Service
pub type SimplePresenceService = PresenceService<UsernamePassword, AuthToken>;
