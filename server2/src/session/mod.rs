use std::time::Instant;

use tracing::log;
use uuid::Uuid;
use xactor::Context;

mod actor;
pub mod command;
pub mod message;

pub type SessionId = Uuid;

pub struct Session {
    pub session_id: SessionId,
    pub connection: Option<xactor::Sender<message::FromSession>>,
    pub last_seen_connected: Instant,
}

impl std::fmt::Debug for Session {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Session")
            .field("session_id", &self.session_id)
            .field("connection is some?", &self.connection.is_some())
            .finish()
    }
}

impl Default for Session {
    fn default() -> Self {
        Session {
            session_id: Uuid::new_v4(),
            connection: None,
            last_seen_connected: Instant::now(),
        }
    }
}

impl Session {
    pub fn with_connection(connection: xactor::Sender<message::FromSession>) -> Self {
        Session {
            connection: Some(connection),
            ..Default::default()
        }
    }

    fn gc(&mut self, ctx: &mut Context<Self>) {
        log::trace!("gc");
        if let Some(can_upgrade) = self.connection.as_ref().map(|c| c.can_upgrade()) {
            if !can_upgrade {
                log::trace!("connection is gone");
                self.connection = None
            } else {
                // I'm still alive, updating timestamp
                self.last_seen_connected = Instant::now();
            }
        } else {
            let secs_since_disconnect = (Instant::now() - self.last_seen_connected).as_secs();
            log::trace!("session without connection {}s", secs_since_disconnect);
            if secs_since_disconnect > 10 {
                log::debug!(
                    "session without connection for more than {}s, stopping session",
                    secs_since_disconnect
                );
                ctx.stop(None);
            }
        }
    }
}
