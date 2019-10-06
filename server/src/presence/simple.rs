//! Simple `PresenceHandler` implementation
//!
//! for simplicity sake this handles user profiles itself, this should probably be handled by another actor

use signaler_protocol as protocol;
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
        created.elapsed() < Duration::from_secs(60 * 2)
    }

    fn grab_profile(&mut self, credentials: &Credentials) -> Option<UserProfile> {
        match credentials {
            Credentials::UsernamePassword {username, password} => {
                if Some(password) == self.user_database.credentials.get(username) {
                    info!("valid login trace {:?}", credentials);

                    let profile = self.user_database.profiles.get(username);

                    if let Some(profile) = profile {
                        trace!("found profile for {:?} -> {:#?}", username, profile);
                    }

                    profile.cloned()
                } else {
                    debug!("not found {:?}", credentials);
                    None
                }
            },
            Credentials::AdHoc{ username } => {
                Some(protocol::UserProfile {full_name: format!("{} (adhoc)", username) }.into())
            }
        }
    }

}


impl PresenceHandler for SimplePresenceHandler {
    type Credentials = Credentials;
    type AuthToken = AuthToken;

    fn associate_user(&mut self, credentials: &Credentials, session_id: &SessionId) -> Option<SimpleAuthResponse> {
        self.clean_up();

        if let Some(profile) = self.grab_profile(credentials) {
            let token = AuthToken::new();
            let session_state = SessionState {
                created: Instant::now(),
                session_id: *session_id
            };
            trace!("currently logged in {:?}", self.running_sessions);

            self.running_sessions.insert(token, session_state); // TODO: prevent clashes
            Some(AuthResponse{ token, profile: profile.clone() })
        } else {
            None
        }

    }

    fn still_valid(&self, token: &AuthToken) -> bool {
        if let Some(session) = self.running_sessions.get(token) {
            Self::still_fresh(session.created)
        } else {
            warn!("{:?} has expired", token);
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
        let clean_up_timeout = Duration::from_secs(5);
        if self.last_update.elapsed() > clean_up_timeout {
            debug!("no cleanup in {:?}", clean_up_timeout);
            self.last_update = Instant::now();
            debug!("cleaning up");
            self.running_sessions = self.running_sessions
                .drain()
                .filter(|(_token, state)| Self::still_fresh(state.created))
                .collect()
        }
    }


}
