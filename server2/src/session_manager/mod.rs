use std::{collections::HashMap, fmt};

use prometheus::IntGauge;
use signaler_protocol::Credentials;
use xactor::{Actor, Addr, Sender};

use crate::session::{Session, SessionId};

mod actor;
pub mod command;

#[derive(Default)]
pub struct SessionManager {
    sessions: HashMap<SessionId, Addr<Session>>,
    open_sessions: Option<IntGauge>,
}

impl fmt::Debug for SessionManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SessionManager")
            .field("sessions", &self.sessions.keys())
            .finish()
    }
}

impl SessionManager {
    pub async fn create_session(
        &mut self,
        _credentials: &Credentials,
        connection: Sender<command::SessionAssociated>,
    ) -> Result<(), anyhow::Error> {
        let session = Session::default();
        let session_id = session.session_id;
        let session_addr = session.start().await?;
        let session_weak = session_addr.downgrade();
        self.sessions.insert(session_id, session_addr);

        connection.send(command::SessionAssociated { session: session_weak })?;

        Ok(())
    }
}
