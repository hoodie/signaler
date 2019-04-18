//! Simple `PresenceHandler` implementation

use super::*;
use crate::session::SessionId;
use crate::static_data::StaticUserDatabase;


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
        created.elapsed() < Duration::from_secs(10 + 5)
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
            trace!("currently logged in {:#?}", self.running_sessions);
            return Some(token);
        } else {
            debug!("not found {:?}", credentials);
        }
        None
    }

    fn still_valid(&self, token: &AuthToken) -> bool {
        if let Some(session) = self.running_sessions.get(token) {
            dbg!(Self::still_fresh(session.created))
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
