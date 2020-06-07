// TODO: merge me with session_manager/mod.rs
//! SessionHandler
//!

use super::*;
use crate::session::SessionId;
use signaler_protocol as protocol;

#[derive(Debug)]
pub struct SessionState {
    created: Instant,
    session_id: SessionId,
    last_update: Instant,
}

#[derive(Debug)]
// TODO: make me mockable by splitting into SessionManager and DefaultSessionManager 
pub struct SessionManager {
    running_sessions: HashMap<AuthToken, SessionState>,
}

impl Default for SessionManager {
    fn default() -> Self {
        SessionManager {
            running_sessions: Default::default(),
        }
    }
}

impl Actor for SessionManager {
    type Context = Context<Self>;
    fn started(&mut self, ctx: &mut Self::Context) {
        debug!("SessionMaanger started");
    }
}
