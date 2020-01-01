use actix::prelude::*;
use actix::WeakAddr;
use signaler_protocol::{ChatMessage, Participant};

use super::{DefaultRoom, RoomId};

#[derive(Message, Debug)]
#[rtype(result = "()")]
pub enum RoomToSession {
    Joined(RoomId, WeakAddr<DefaultRoom>),
    ChatMessage {
        room: RoomId,
        message: ChatMessage,
    },

    History {
        room: RoomId,
        messages: Vec<ChatMessage>,
    },

    RoomState {
        room: RoomId,
        roster: Vec<Participant>,
    },

    JoinDeclined {
        room: RoomId,
    },

    Left {
        room: RoomId,
    },
}
