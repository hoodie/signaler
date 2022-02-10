use hannibal::WeakAddr;
use signaler_protocol::{self as protocol, ChatMessage, RoomId};

use crate::session::SessionId;

use super::{participant::RoomParticipant, Room};

#[derive(Debug)]
#[hannibal::message]
pub enum Command {
    AddParticipant {
        participant: RoomParticipant,
    },

    // GetParticipants {
    //     session_id: SessionId,
    // },
}

#[derive(Debug)]
#[hannibal::message]
pub struct ChatRoomCommand {
    pub command: protocol::ChatRoomCommand,
    pub session_id: SessionId,
}

#[derive(Debug)]
#[hannibal::message]
pub enum RoomToSession {
    Joined(RoomId, WeakAddr<Room>),

    ChatMessage { room: RoomId, message: ChatMessage },
    // History { room: RoomId, messages: Vec<ChatMessage> },

    // RoomState { room: RoomId, roster: Vec<Participant> },

    // RoomEvent { room: RoomId, event: RoomEvent },

    // JoinDeclined { room: RoomId },

    // Left { room: RoomId },
}
