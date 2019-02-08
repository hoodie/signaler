//! Public API
//!
//! these are messages the http client can send via a [ClientSession](../session/struct.ClientSession.html)

use serde::{Deserialize, Serialize};
use crate::session::ClientSession;
use crate::server::{RoomId, SessionId};
use crate::user_management::{UsernamePassword, UserProfile};

/// Actual chat Message
///
/// is send via `SessionCommand::Message` and received via `SessionMessage::Message`
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatMessage {
    pub content: String,
    pub sender: String,
}

impl ChatMessage {
    pub fn new(content: String, sender: SessionId) -> Self {
        Self {
            content,
            sender: sender.to_string()
        }
    }
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
    Welcome { session: ClientSession },

    Authenticated { profile: UserProfile },

    RoomList { rooms: Vec<String> },

    MyRoomList { rooms: Vec<String> },

    Message { message: ChatMessage, room: RoomId},

    Any { payload: serde_json::Value },

    Ok, // 200

    Error { message: String },
}

impl SessionMessage {
    pub fn err(msg: impl Into<String>) -> Self {
        SessionMessage::Error{ message: msg.into()}
    }

    pub fn to_json(self) -> String {
        serde_json::to_string(&self).unwrap()
    }

    /// dev convenience only!
    pub fn any<T: serde::Serialize>(anything: T) -> Self {
        SessionMessage::Any {
            payload: serde_json::to_value(&anything).unwrap()
        }
    }

}