use std::collections::HashMap;

use signaler_protocol::ChatMessage;
pub use signaler_protocol::RoomId;

use tracing::log;
use uuid::Uuid;

mod actor;
mod command;
pub use command::Command;

use crate::session::SessionId;

use self::participant::RoomParticipant;

pub mod participant;

#[derive(Debug)]
pub struct Room {
    id: RoomId,
    ephemeral: bool,
    history: Vec<ChatMessage>,
    roster: HashMap<SessionId, RoomParticipant>,
}

impl Room {
    pub fn new(id: RoomId) -> Self {
        Self {
            id,
            ephemeral: true,
            history: Vec::with_capacity(10_000),
            roster: Default::default(),
        }
    }

    pub fn permanent(id: RoomId) -> Self {
        Self {
            id,
            ephemeral: false,
            history: Vec::with_capacity(10_000),
            roster: Default::default(),
        }
    }

    pub fn add_participant(&mut self, participant: RoomParticipant) {
        if let Some(old) = self.roster.insert(participant.session_id, participant) {
            log::warn!("replacing existing an participant {:?}", old)
        }
        log::debug!("room {:?} has {} participants", self.id, self.roster.len())
    }
}
