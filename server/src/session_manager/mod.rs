use std::{collections::HashMap, fmt};

use prometheus::IntGauge;
use signaler_protocol::Credentials;
use tracing::log;
use hannibal::{Actor, Addr, Context, WeakAddr};

use crate::{
    connection::Connection,
    session::{Session, SessionId},
};

mod actor;
pub mod command;

#[derive(Default)]
pub struct SessionManager {
    sessions: HashMap<SessionId, Addr<Session>>,
    open_sessions: Option<IntGauge>,
}

impl fmt::Debug for SessionManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SessionManager")
            .field("sessions", &self.sessions.keys())
            .finish()
    }
}

impl SessionManager {
    pub async fn create_session(
        &mut self,
        _credentials: &Credentials,
        // connection: Sender<command::SessionAssociated>,
        connection: WeakAddr<Connection>,
    ) -> Result<(), anyhow::Error> {
        if let Some(connection) = connection.upgrade() {
            let session = Session::with_connection(connection.sender());
            let session_id = session.session_id;
            let session_addr = session.start().await?;
            let session_weak = session_addr.downgrade();
            self.sessions.insert(session_id, session_addr);

            connection.send(command::SessionAssociated { session: session_weak })?;
        } else {
            anyhow::bail!("connection is already dead")
        }

        Ok(())
    }

    fn gc(&mut self, _ctx: &mut Context<Self>) {
        // log::trace!("gc");
        self.sessions.retain(|id, session| {
            if session.stopped() {
                log::trace!("session {} has stopped", id);
                if let Some(gauge) = self.open_sessions.as_ref() {
                    gauge.dec();
                    log::trace!("decreasing sessions count {:?}", gauge.get());
                }
                false
            } else {
                true
            }
        });
    }
}
