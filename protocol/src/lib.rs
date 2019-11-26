//! Public API
//!
//! these are messages the http client can send via a [ClientSession](../session/struct.ClientSession.html)

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[cfg(target_arch="wasm32")]
extern crate wasm_bindgen;

pub type SessionId = Uuid;
pub type RoomId = String;


#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum Credentials {
    /// Simple Authentication Credentials
    UsernamePassword {
        username: String,
        password: String,
    },

    /// Even simpler Authentication Credentials
    AdHoc {
        username: String,
    }
}


#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserProfile {
    pub full_name: String,
}




/// Actual chat Message
///
/// is send via `SessionCommand::Message` and received via `SessionMessage::Message`
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatMessage {
    pub content: String,
    pub sender: SessionId,
    pub sender_name: String,
    pub sent: chrono::DateTime<chrono::Utc>,
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
            sent: chrono::Utc::now(),
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

    /// Join a particular room
    Leave { room: RoomId },

    /// Send a message to all participants of that room
    Message { message: String, room: RoomId },

    /// List all rooms
    ListRooms,

    /// List rooms I'm member of
    ListMyRooms,

    ListParticipants { room: RoomId },

    /// shutdown server 😈
    ShutDown,

    /// Request Authentication Token
    Authenticate { credentials: Credentials }
}

impl SessionCommand {
    pub fn suggestions() -> String {
        use SessionCommand::*;
        let room = "roomName";
        serde_json::to_string_pretty(&[
            Join { room: room.into() },
            Message { message:  "hello world".into(), room: room.into() },
            Authenticate {credentials: Credentials::UsernamePassword {
                username: "username".into(),
                password: "password".into(),
            }},
            ListRooms,
            ListMyRooms,
            ShutDown,
        ]).unwrap()
    }
}

/// Message received from the server
#[derive(Debug, Serialize, Deserialize)]
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

#[cfg(target_arch="wasm32")]
mod wasm_wrapper {
    #![allow(unused_macros, unused_imports)]
    use wasm_bindgen::prelude::*;
    use wasm_bindgen::JsCast;
    use serde_json;

    use crate::SessionMessage;

    #[wasm_bindgen]
    extern {
        #[wasm_bindgen(js_namespace = console)] pub fn warn(msg: &str);
        #[wasm_bindgen(js_namespace = console)] pub fn debug(msg: &str);
        #[wasm_bindgen(js_namespace = console)] pub fn error(msg: &str);
    }

    macro_rules! debug { ($($arg:tt)*) => (debug(&format!($($arg)*));) }
    macro_rules! warn { ($($arg:tt)*) => (warn(&format!($($arg)*));) }
    macro_rules! error { ($($arg:tt)*) => (error(&format!($($arg)*));) }


    #[wasm_bindgen]
    pub fn dispatch_message(raw: &JsValue) {
        use SessionMessage::*;
        let msg: SessionMessage = raw.into_serde().unwrap();
        match msg {
            Welcome { session } => debug!("welcome {:?}", session),
            Authenticated => debug!(r"Authenticated \0/"),
            Profile { profile } => debug!("profile: {:?}", profile),
            RoomList { rooms } => debug!("RoomsList: {:?}", rooms),
            MyRoomList { rooms } => debug!("MyRoomList: {:?}", rooms),
            RoomParticipants { room, participants} => debug!("RoomParticipants of {:?}: {:?}", room, participants),
            Message { message, room} => debug!("Message in {room:?} {message:?}", room = room, message = message ),
            Any { payload } => debug!("Any: {:#?}", payload),
            Error { message } => debug!("Error: {}", message),
        }
    }
}