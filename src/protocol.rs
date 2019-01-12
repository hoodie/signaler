
pub mod public {
    use serde_derive::{Deserialize, Serialize};

    /// Message Format
    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct SignalMessage {
        pub kind: SignalKind,
    }

    impl SignalMessage {
        pub fn ok() -> Self {
            Self::from(SignalKind::Ok)
        }

        pub fn join() -> Self {
            Self::from(SignalKind::Join{
                channel: String::from("default"),
            })
        }

        pub fn list() -> Self {
            SignalKind::ListRooms.into()
        }

        pub fn err(msg: impl Into<String>) -> Self {
            SignalKind::Err(msg.into()).into()
        }

        /// dev convenience only!
        pub fn any<T: serde::Serialize>(anything: T) -> Self {
            Self::from(SignalKind::Any(serde_json::to_value(&anything).unwrap()))
        }

        pub fn to_json(self) -> String {
            serde_json::to_string(&self).unwrap()
        }
    }

    impl From<SignalKind> for SignalMessage {
        fn from(kind: SignalKind) -> Self {
            SignalMessage { kind }
        }
    }

    /// Message Format
    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub enum SignalKind {
        Join { channel: String },
        Any ( serde_json::Value ),
        ListRooms,
        ShutDown,
        Ok, // 200
        Err(String),
    }
}

pub mod internal {
    use actix::prelude::*;

    // #[derive(Message)]
    // #[rtype(result = "Vec<String>")]
    pub struct ListRooms;

    impl actix::Message for ListRooms {
        type Result = Vec<String>;
    }
}