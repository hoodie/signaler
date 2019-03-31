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

use std::time::{Duration, Instant};
use std::collections::HashMap;

pub use simple::SimplePresenceService;

mod simple {
    //! Simple Implementation

    use super::*;
    use crate::server::SessionId;
    use crate::static_data::StaticUserDatabase;

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


    #[derive(Debug)]
    pub struct SimplePresenceHandler {
        user_database: StaticUserDatabase,
        running_sessions: HashMap<AuthToken, SessionState>,
        last_update: Instant,
    }

    impl SimplePresenceHandler {
        pub fn new() -> Self {
            Self {
                user_database: StaticUserDatabase::load(),
                last_update: Instant::now(),
                running_sessions: Default::default()
            }
        }

        fn still_fresh(created: Instant) -> bool {
            created.elapsed() < Duration::from_secs(30 * 5)
        }

    }

    impl PresenceHandler for SimplePresenceHandler {
        type Credentials = UsernamePassword;
        type AuthToken = AuthToken;

        fn associate_user(&mut self, credentials: &Self::Credentials, session_id: &SessionId) -> Option<Self::AuthToken> {
            let UsernamePassword {username, password} = credentials;

            let clean_up_timeout = Duration::from_secs(5);
            if self.last_update.elapsed() > clean_up_timeout {
                debug!("no cleanup in {:?}", clean_up_timeout);
                self.last_update = Instant::now();
                self.clean_up();
            }

            if Some(password) == self.user_database.credentials.get(username) {
                let token = AuthToken::new();
                info!("valid login trace {:?} -> {:?}", credentials, token);
                self.running_sessions.insert(token, SessionState {
                    created: Instant::now(),
                    session_id: *session_id
                });
                trace!("currently logged in {:?}", self.running_sessions);
                return Some(token);
            } else {
                debug!("not found {:?}", credentials);
            }
            None
        }

        fn still_valid(&self, token: &AuthToken) -> bool {
            if let Some(session) = self.running_sessions.get(token) {
                Self::still_fresh(session.created)
            } else {
                false
            }
        }

        fn refresh(&mut self, token: &AuthToken) -> Option<AuthToken> {
            if let Some(state) = self.running_sessions.get_mut(token) {
                state.created = Instant::now();
                Some(*token)
            } else {
                None
            }
        }

        fn logout(&mut self, token: &AuthToken) -> bool {
            self.running_sessions.remove(token).is_some()
        }

        fn clean_up(&mut self) {
            debug!("cleaning up");
            self.running_sessions = self.running_sessions
                .drain()
                .filter(|(_token, state)| Self::still_fresh(state.created))
                .collect()
        }

    }

    impl Default for SimplePresenceService {
        fn default() -> Self {
            PresenceService::simple()
        }
    }

    impl Actor for SimplePresenceService {
        type Context = Context<Self>;
        fn started(&mut self, _ctx: &mut Self::Context) {
            debug!("presence started");
        }
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

    impl Handler<ValidateRequest> for SimplePresenceService {
        type Result = MessageResult<ValidateRequest>;
        fn handle(&mut self, request: ValidateRequest, _ctx: &mut Self::Context) -> Self::Result {
            let ValidateRequest {token} = request;
            MessageResult(self.still_valid(&token))
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
#[derive(Message)]
#[rtype(result = "bool")]
pub struct ValidateRequest {
    pub token: AuthToken,
}