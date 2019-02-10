//! Presence Module
//!
//! PresenceService Actor that keeps track of which users are logged in and have which status at a time.
//! Work in progress...
//!

use actix::prelude::*;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

use super::*;
use crate::server::SessionId;

use std::time::Instant;
use std::collections::HashMap;

pub use simple::SimplePresenceService;

mod simple {
    //! Simple Implementation

    use super::*;
    use crate::server::SessionId;
    use crate::user_management::NaiveUserDatabase;

    /// Simple Presence Service
    pub type SimplePresenceService = PresenceService<UsernamePassword, AuthToken>;

    impl SimplePresenceService {
        pub fn new() -> Self {
            super::PresenceService::simple()
        }
    }

    #[derive(Debug)]
    pub struct SessionState {
        created: Instant,
        session_id: SessionId
    }


    #[derive(Default, Debug)]
    pub struct SimplePresenceHandler {
        user_database: NaiveUserDatabase,
        running_sessions: HashMap<AuthToken, SessionState>
    }

    impl SimplePresenceHandler {
        pub fn new() -> Self {
            Self {
                user_database: NaiveUserDatabase::load(),
                .. Default::default()
            }
        }
    }

    impl PresenceHandler for SimplePresenceHandler {
        type Credentials = UsernamePassword;
        type AuthToken = AuthToken;

        fn associate_user(&mut self, credentials: &Self::Credentials, session_id: &SessionId) -> Option<Self::AuthToken> {
            let UsernamePassword {username, password} = credentials;

            if Some(password) == self.user_database.credentials.get(username) {
                let token = AuthToken::new();
                info!("valid login trace {:?} -> {:?}", credentials, token);
                self.running_sessions.insert(token, SessionState {
                    created: Instant::now(),
                    session_id: *session_id
                });
                trace!("currently logged in {:#?}", self.running_sessions);
                return Some(token);
            } else {
                debug!("not found {:?}", credentials);
            }
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
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct AuthToken(Uuid);
impl AuthToken {
    pub fn new() -> Self {
        Self (Uuid::new_v4())
    }
}

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