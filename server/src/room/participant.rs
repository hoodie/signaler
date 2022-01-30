use xactor::WeakAddr;

use crate::session::{Session, SessionId};

pub struct RoomParticipant {
    pub session_id: SessionId,
    pub addr: WeakAddr<Session>,
    pub profile: String,
}

impl std::fmt::Debug for RoomParticipant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RoomParticipant")
            .field("session_id", &self.session_id)
            .field("addr", &"???".to_string())
            .field("profile", &self.profile)
            .finish()
    }
}
