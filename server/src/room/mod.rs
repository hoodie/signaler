use std::collections::{HashMap, VecDeque};

use signaler_protocol::ChatMessage;
pub use signaler_protocol::RoomId;

use hannibal::Context;
use tracing::log;

mod actor;
pub mod command;
pub use command::Command;

use crate::{room::command::RoomToSession, session::SessionId};

use self::participant::RoomParticipant;

pub mod participant;

#[derive(Debug)]
pub struct Room {
    id: RoomId,
    capacity: usize,
    history: VecDeque<ChatMessage>,
    roster: HashMap<SessionId, RoomParticipant>,
}

impl Room {
    pub fn new(id: RoomId) -> Self {
        let capacity: usize = 10_000;
        Self {
            id,
            capacity,
            history: VecDeque::with_capacity(capacity),
            roster: Default::default(),
        }
    }

    pub fn add_participant(&mut self, participant: RoomParticipant, ctx: &mut Context<Self>) {
        if let Some(ref participant_addr) = participant.addr.upgrade() {
            if let Some(old) = self.roster.insert(participant.session_id, participant) {
                log::warn!("replacing existing an participant {:?}", old)
            }
            if let Err(error) = participant_addr.send(RoomToSession::Joined(self.id.clone(), ctx.address().downgrade()))
            {
                log::warn!("failed to send Joined {error}");
            }
        }
        log::debug!("room {:?} has {} participants", self.id, self.roster.len())
    }

    pub fn forward_to_participants(&mut self, message: ChatMessage, _ctx: &mut Context<Self>) {
        self.store_message(&message);
        for participant in self.roster.iter().filter_map(|(_, p)| p.addr.upgrade()) {
            participant
                .send(RoomToSession::ChatMessage {
                    room: self.id.clone(),
                    message: message.clone(),
                })
                .unwrap()
        }
    }

    fn store_message(&mut self, message: &ChatMessage) {
        if self.history.len() == self.capacity {
            self.history.pop_front();
        }
        self.history.push_back(message.to_owned());
    }
}
