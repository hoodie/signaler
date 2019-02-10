//! Presence Module
//! 
//! PresenceService Actor that keeps track of which users are logged in and have which status at a time.
//! Work in progress...
//! 

use actix::prelude::*;
use serde::{Serialize, Deserialize};

use super::*;
use crate::server::SessionId;

use std::time::Instant;
use std::collections::HashMap;

pub use simple::SimplePresenceService;

mod simple {
    //! Simple Implementation

    use super::*;
    use crate::server::SessionId;

    /// Simple Presence Service
    pub type SimplePresenceService = PresenceService<UsernamePassword, AuthToken>;

    impl SimplePresenceService {
        pub fn new() -> Self {
            super::PresenceService::simple()
        }
    }

    #[derive(Debug)]
    pub struct SessionState {
        pub last_checking: Instant
    }


    #[derive(Default, Debug)]
    pub struct SimplePresenceHandler {
        running_sessions: HashMap<SessionId, SessionState>
    }

    impl PresenceHandler for SimplePresenceHandler {
        type Credentials = UsernamePassword;
        type AuthToken = AuthToken;

        fn associate_user(&mut self, credentials: &Self::Credentials, session_id: &SessionId) -> Option<Self::AuthToken> {
            None
        }

    }

    impl Default for SimplePresenceService {
        fn default() -> Self {
            PresenceService::simple()
        }
    }

    impl Actor for SimplePresenceService {
        type Context = Context<Self>;
    }

    impl SystemService for SimplePresenceService {}
    impl Supervised for SimplePresenceService {}

    /// implementation docs
    impl Handler<AuthenticationRequest<UsernamePassword>> for SimplePresenceService {
        type Result = MessageResult<AuthenticationRequest<UsernamePassword>>;

        fn handle(&mut self, request: AuthenticationRequest<UsernamePassword>, _ctx: &mut Self::Context) -> Self::Result {
            info!("received AuthenticationRequest");

            let AuthenticationRequest {credentials, session_id} = request;

            MessageResult(self.associate_user(&credentials, &session_id))
        }
    }

}

/// Simple Authentication Credentials
#[derive(Debug, Serialize, Deserialize)]
pub struct UsernamePassword {
    pub username: String,
    pub password: String,
}

/// Token returned after successful authentication
/// 
/// Use this to make requests that require authentication. Should timeout.
#[derive(Debug, Serialize, Deserialize)]
pub struct AuthToken ();

/// General Behaviour of a PresenceService
pub trait PresenceHandler {
    type Credentials;
    type AuthToken;

    fn associate_user(&mut self, credentials: &Self::Credentials, session_id: &SessionId) -> Option<Self::AuthToken>;

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
}

impl PresenceService<UsernamePassword, AuthToken> {
    pub fn simple() -> simple::SimplePresenceService {
        Self {
            inner: Box::new(simple::SimplePresenceHandler::default())
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