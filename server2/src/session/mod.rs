use uuid::Uuid;

mod actor;
pub mod command;
pub mod message;

pub type SessionId = Uuid;

pub struct Session {
    pub session_id: SessionId,
    pub connection: Option<xactor::Sender<message::FromSession>>,
}

impl Session {
    pub fn with_connection(connection:xactor::Sender<message::FromSession> ) -> Self {
        Session {
            connection: Some(connection),
            ..Default::default()
        }
    }
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
        }
    }
}

trait GarbageCollector {
    type Managed;

    fn gc(&mut self);
}

impl GarbageCollector for Session {
    type Managed = Option<xactor::Sender<message::FromSession>>;
    fn gc(&mut self) {
        if let Some(can_upgrade) = self.connection.as_ref().map(|s| s.can_upgrade()) {
            if !can_upgrade {
                self.connection = None
            }
        }
    }
}
