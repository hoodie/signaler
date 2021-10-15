use uuid::Uuid;

mod actor;
pub mod command;

pub type SessionId = Uuid;

#[derive(Debug)]
pub struct Session {
    pub session_id: SessionId,
}

impl Default for Session {
    fn default() -> Self {
        Session {
            session_id: Uuid::new_v4(),
        }
    }
}
