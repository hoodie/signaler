
pub mod public {
    use serde_derive::{Deserialize, Serialize};
    use crate::session::ClientSession;

    /// Message Format
    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct SessionCommand {
        pub kind: SessionCommandKind,
    }

    /// Message Format
    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub enum SessionCommandKind {
        Join { room: String },
        ListRooms,
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

    /// Message Format
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
            Self::from(SessionMessageKind::Any(serde_json::to_value(&anything).unwrap()))
        }

        pub fn err(msg: impl Into<String>) -> Self {
            SessionMessageKind::Err(msg.into()).into()
        }

        pub fn to_json(self) -> String {
            serde_json::to_string(&self).unwrap()
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

    #[derive(Message)]
    #[rtype(result = "Vec<String>")]
    pub struct ListRooms;

    #[derive(Message)]
    #[rtype(result = "()")]
    pub struct Ping;

    #[derive(Message)]
    #[rtype(result = "()")]
    pub struct ServerToSession;

    #[derive(Message)]
    #[rtype(result = "()")]
    pub struct JoinRoom {
        pub room: String,
        pub uuid: Uuid,
        pub addr: Recipient<ServerToSession>,
    }

}