//! Public API
//!
//! these are messages the http client can send via a [ClientSession](../session/struct.ClientSession.html)

use serde_derive::{Deserialize, Serialize};
use crate::session::ClientSession;
use crate::server::RoomId;

/// Command sent to the server
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionCommand {
    pub kind: SessionCommandKind,
}

/// Actual chat Message
/// 
/// is send via `SessionCommandKind::Message` and received via `SessionMessageKind::Message`
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatMessage {
    pub content: String
}

/// Message Format
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SessionCommandKind {
    /// Join a particular room
    Join { room: RoomId },
    /// Send a message to all participants of that room
    Message { message: ChatMessage, room: RoomId},
    /// List all rooms
    ListRooms,
    /// List rooms I'm member of
    ListMyRooms,
    /// shutdown server 😈
    ShutDown,
}

impl SessionCommand {

    pub fn join() -> Self {
        Self::from(SessionCommandKind::Join{
            room: String::from("default"),
        })
    }

    pub fn list() -> Self {
        SessionCommandKind::ListRooms.into()
    }

    pub fn to_json(self) -> String {
        serde_json::to_string(&self).unwrap()
    }
}

/// Message received from the server
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionMessage {
    pub kind: SessionMessageKind,
}

/// Message Format
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SessionMessageKind {
    Welcome { session: ClientSession },
    RoomList(Vec<String>),
    Message { message: ChatMessage, room: RoomId},
    Any ( serde_json::Value ),
    Ok, // 200
    Err(String),
}

impl SessionMessage {
    pub fn ok() -> Self {
        Self::from(SessionMessageKind::Ok)
    }

    /// dev convenience only!
    pub fn any<T: serde::Serialize>(anything: T) -> Self {
        Self::from(SessionMessageKind::any(&anything))
    }

    pub fn err(msg: impl Into<String>) -> Self {
        SessionMessageKind::Err(msg.into()).into()
    }

    pub fn to_json(self) -> String {
        serde_json::to_string(&self).unwrap()
    }
}

impl SessionMessageKind {
    /// dev convenience only!
    pub fn any<T: serde::Serialize>(anything: T) -> Self {
        SessionMessageKind::Any(serde_json::to_value(&anything).unwrap())
    }

}


impl From<SessionMessageKind> for SessionMessage {
    fn from(kind: SessionMessageKind) -> Self {
        SessionMessage { kind }
    }
}

impl From<SessionCommandKind> for SessionCommand {
    fn from(kind: SessionCommandKind) -> Self {
        SessionCommand { kind }
    }
}

