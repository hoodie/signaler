//! Public API
//!
//! these are messages the http client can send via a [ClientSession](../session/struct.ClientSession.html)

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::session::SessionId;
use crate::room::RoomId;
use crate::presence::UsernamePassword;
use crate::user_management::UserProfile;

/// Actual chat Message
///
/// is send via `SessionCommand::Message` and received via `SessionMessage::Message`
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatMessage {
    pub content: String,
    pub sender: SessionId,
    pub sender_name: String,
    pub uuid: Uuid,
}

/// Actual chat Message
///
/// is send via `SessionCommand::Message` and received via `SessionMessage::Message`
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Participant {
    pub full_name: String,
    pub session_id: SessionId,
}

impl From<(UserProfile, SessionId)> for Participant {
    fn from((profile, session_id): (UserProfile, SessionId)) -> Participant {
        Participant {
            full_name: profile.full_name,
            session_id,
        }
    }
}


impl ChatMessage {
    pub fn new(content: String, sender: SessionId, sender_name: &str) -> Self {
        Self {
            content,
            sender,
            sender_name: sender_name.into(),
            uuid: Uuid::new_v4(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionDescription {
    pub session_id: SessionId,
}

/// Command sent to the server
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum SessionCommand {
    /// Join a particular room
    Join { room: RoomId },

    /// Send a message to all participants of that room
    Message { message: String , room: RoomId},

    /// List all rooms
    ListRooms,

    /// List rooms I'm member of
    ListMyRooms,

    /// shutdown server 😈
    ShutDown,

    /// Request Authentication Token
    Authenticate { credentials: UsernamePassword }
}

impl SessionCommand {
    pub fn suggestions() -> String {
        use SessionCommand::*;
        let room = "roomName";
        serde_json::to_string_pretty(&[
            Join { room: room.into() },
            Message { message:  "hello world".into(), room: room.into() },
            ListRooms,
            ListMyRooms,
            ShutDown,
        ]).unwrap()
    }
}

/// Message received from the server
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum SessionMessage {
    Welcome { session: SessionDescription },

    /// response to `SessionCommand::Authenticate`
    Authenticated,

    Profile { profile: UserProfile },

    RoomList { rooms: Vec<String> },

    MyRoomList { rooms: Vec<String> },

    RoomParticipants { room: RoomId, participants: Vec<Participant>},

    Message { message: ChatMessage, room: RoomId},

    Any { payload: serde_json::Value },

    Error { message: String },
}

impl SessionMessage {
    pub fn err(msg: impl Into<String>) -> Self {
        SessionMessage::Error{ message: msg.into()}
    }

    pub fn into_json(self) -> String {
        serde_json::to_string(&self).unwrap()
    }

    /// dev convenience only!
    pub fn any<T: serde::Serialize>(anything: T) -> Self {
        SessionMessage::Any {
            payload: serde_json::to_value(&anything).unwrap()
        }
    }

}