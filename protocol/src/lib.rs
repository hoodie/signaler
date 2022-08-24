//! Public API
//!
//! these are messages the http client can send via a [ClientSession](../session/struct.ClientSession.html)

use serde::{Deserialize, Serialize};
use typescript_definitions::TypeScriptify;
use uuid::Uuid;

#[cfg(target_arch = "wasm32")]
extern crate wasm_bindgen;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SessionId(Uuid);

impl From<Uuid> for SessionId {
    fn from(inner: Uuid) -> Self {
        Self(inner)
    }
}

#[derive(Clone, Hash, PartialEq, Eq, Debug, Serialize, Deserialize, TypeScriptify)]
pub struct RoomId(String);

impl<T: Into<String>> From<T> for RoomId {
    fn from(inner: T) -> Self {
        Self(inner.into())
    }
}

impl std::fmt::Display for RoomId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::ops::Deref for RoomId {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// impl TypeScriptifyTrait for SessionId {
//     fn type_script_ify() -> std::borrow::Cow<'static, str> {
//         "\n".into()
//     }
// }

#[derive(Clone, Debug, Serialize, Deserialize, TypeScriptify)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum Credentials {
    /// Simple Authentication Credentials
    UsernamePassword { username: String, password: String },

    /// Even simpler Authentication Credentials
    AdHoc { username: String },
    // TODO: Token { token: Uuid }
}

#[derive(Clone, Debug, Serialize, Deserialize, TypeScriptify)]
#[serde(rename_all = "camelCase")]
pub struct UserProfile {
    pub full_name: String,
}

/// Actual chat Message
///
/// is send via `SessionCommand::Message` and received via `SessionMessage::Message`
#[derive(Clone, Debug, Serialize, Deserialize, TypeScriptify)]
#[serde(rename_all = "camelCase")]
pub struct ChatMessage {
    pub content: String,
    pub sender: SessionId,
    pub sent: chrono::DateTime<chrono::Utc>,
    pub uuid: Uuid,
}

impl ChatMessage {
    pub fn new(content: String, sender: SessionId) -> Self {
        Self {
            content,
            sender,
            sent: chrono::Utc::now(),
            uuid: Uuid::new_v4(),
        }
    }
}

/// SessionId and Full Name
///
/// is send via `SessionCommand::Message` and received via `SessionMessage::Message`
#[derive(Clone, Debug, Serialize, Deserialize, TypeScriptify)]
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

#[derive(Clone, Debug, Serialize, Deserialize, TypeScriptify)]
#[serde(rename_all = "camelCase")]
pub enum RoomEvent {
    ParticipantJoined { name: String },
    ParticipantLeft { name: String },
}

/// Command sent to the server
#[derive(Clone, Debug, Serialize, Deserialize, TypeScriptify)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum ConnectionCommand {
    /// Request Authentication Token
    Authenticate { credentials: Credentials },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionDescription {
    pub session_id: Uuid,
}

/// Command sent to the server
#[derive(Debug, Serialize, Deserialize, TypeScriptify)]
#[serde(rename_all = "camelCase", tag = "type")]
#[rustfmt::skip]
pub enum SessionCommand {
    /// Join a particular room
    Join { room: RoomId },

    /// Send a message to all participants of that room
    ChatRoom { room: RoomId, command: ChatRoomCommand },

    /// List all rooms
    ListRooms,

    /// List rooms I'm member of
    ListMyRooms,

    /// shutdown server 😈
    ShutDown,

    /// Request Authentication Token
    Authenticate { credentials: Credentials },
}

/// Command sent to the server
#[derive(Debug, Serialize, Deserialize, TypeScriptify)]
#[serde(rename_all = "camelCase", tag = "type")]
#[rustfmt::skip]
pub enum ChatRoomCommand {
    /// Leave a particular room
    Leave,

    /// Send a message to all participants of that room
    Message { content: String },

    ListParticipants,
}

impl SessionCommand {
    pub fn suggestions() -> String {
        use SessionCommand::*;
        let room = "roomName";
        serde_json::to_string_pretty(&[
            Join { room: room.into() },
            Authenticate {
                credentials: Credentials::UsernamePassword {
                    username: "username".into(),
                    password: "password".into(),
                },
            },
            ListRooms,
            ListMyRooms,
            ShutDown,
        ])
        .unwrap()
    }
}

pub trait SessionCommandDispatcher {
    type Context;

    fn dispatch_command(&self, msg: SessionCommand, ctx: &mut Self::Context);
}

/// Message received from the server
#[derive(Debug, Serialize, Deserialize, TypeScriptify)]
#[serde(rename_all = "camelCase", tag = "type")]
#[rustfmt::skip]
pub enum SessionMessage {
    Welcome { session: SessionDescription },

    /// response to `SessionCommand::Authenticate`
    Authenticated,

    Profile { profile: UserProfile },

    RoomList { rooms: Vec<String> },

    MyRoomList { rooms: Vec<String> },

    RoomParticipants { room: RoomId, participants: Vec<Participant> },
    RoomEvent { room: RoomId, event: RoomEvent },

    Message { message: ChatMessage, room: RoomId },

    Any { payload: serde_json::Value },

    Error { message: String },
}

impl SessionMessage {
    pub fn err(msg: impl Into<String>) -> Self {
        SessionMessage::Error { message: msg.into() }
    }

    pub fn into_json(self) -> String {
        serde_json::to_string(&self).unwrap()
    }

    /// dev convenience only!
    pub fn any<T: serde::Serialize>(anything: T) -> Self {
        SessionMessage::Any {
            payload: serde_json::to_value(&anything).unwrap(),
        }
    }
}

#[cfg(target_arch = "wasm32")]
mod wasm_wrapper {
    use serde_json;
    use wasm_bindgen::prelude::*;
    use wasm_bindgen::JsCast;

    use crate::SessionMessage;

    #[wasm_bindgen]
    #[rustfmt::skip]
    extern "C" {
        #[wasm_bindgen(js_namespace = console)] pub fn warn(msg: &str);
        #[wasm_bindgen(js_namespace = console)] pub fn debug(msg: &str);
        #[wasm_bindgen(js_namespace = console)] pub fn error(msg: &str);
    }

    macro_rules! debug { ($($arg:tt)*) => (debug(&format!($($arg)*));) }
    macro_rules! warn { ($($arg:tt)*) => (warn(&format!($($arg)*));) }
    macro_rules! error { ($($arg:tt)*) => (error(&format!($($arg)*));) }

    #[wasm_bindgen]
    #[rustfmt::skip]
    pub fn dispatch_message(raw: &JsValue) {
        use SessionMessage::*;
        let msg: SessionMessage = raw.into_serde().unwrap();
        match msg {
            Welcome { session } => log::debug!("welcome {:?}", session),
            Authenticated => log::debug!(r"Authenticated \0/"),
            Profile { profile } => log::debug!("profile: {:?}", profile),
            RoomList { rooms } => log::debug!("RoomsList: {:?}", rooms),
            MyRoomList { rooms } => log::debug!("MyRoomList: {:?}", rooms),
            RoomParticipants { room, participants } => log::debug!("RoomParticipants of {:?}: {:?}", room, participants),
            RoomEvent {room, event } => log::debug!("{room:?} {event:#?}"),
            Message { message, room } => log::debug!( "Message in {room:?} {message:?}", room = room, message = message),
            Any { payload } => log::debug!("Any: {:#?}", payload),
            Error { message } => log::debug!("Error: {}", message),
        }
    }
}
