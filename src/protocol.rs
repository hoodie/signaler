
pub mod public {
    use serde_derive::{Deserialize, Serialize};
    use crate::session::ClientSession;
    use super::internal::RoomId;

    /// Command to the server
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
        content: String
    }

    /// Message Format
    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub enum SessionCommandKind {
        Join { room: RoomId },
        Message { message: ChatMessage, room: RoomId},
        ListRooms,
        ListMyRooms,
        ShutDown,
        Err(String),
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

        pub fn err(msg: impl Into<String>) -> Self {
            SessionCommandKind::Err(msg.into()).into()
        }

        pub fn to_json(self) -> String {
            serde_json::to_string(&self).unwrap()
        }
    }

    /// Message from the server
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

}

pub mod internal {
    use actix::prelude::*;
    use uuid::Uuid;
    use super::public::ChatMessage;

    pub type RoomId = String;

    #[derive(Message)]
    #[rtype(result = "Vec<String>")]
    pub struct ListRooms;

    #[derive(Message)]
    #[rtype(result = "Vec<String>")]
    pub struct ListMyRooms {
        pub uuid: Uuid,
    }

    #[derive(Message)]
    #[rtype(result = "()")]
    pub struct Ping;

    #[derive(Message, Debug)]
    #[rtype(result = "()")]
    pub enum ServerToSession {
        Forward(ChatMessage, RoomId),
        ChatMessage(ChatMessage),
    }

    #[derive(Message)]
    #[rtype(result = "Result<(), String>")]
    pub struct JoinRoom {
        pub room: String,
        pub uuid: Uuid,
        pub addr: Recipient<ServerToSession>,
    }

    #[derive(Message)]
    #[rtype(result = "()")]
    pub struct LeaveRoom {
        pub room: String,
        pub uuid: Uuid,
        pub addr: Recipient<ServerToSession>,
    }

    #[derive(Message)]
    #[rtype(result = "()")]
    pub struct LeaveAllRooms {
        pub room: String,
        pub uuid: Uuid,
        pub addr: Recipient<ServerToSession>,
    }

}
