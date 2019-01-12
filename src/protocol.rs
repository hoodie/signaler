use serde_derive::{Serialize, Deserialize};

/// Message Format
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignalMessage {
    pub kind: SignalKind,
}

impl SignalMessage {
    pub fn ok() -> Self { SignalMessage { kind: SignalKind::Ok} }
    pub fn join() -> Self { SignalMessage { kind: SignalKind::Join } }
    pub fn list() -> Self { SignalMessage { kind: SignalKind::List } }
    pub fn err(msg: impl Into<String>) -> Self {
        SignalMessage {
            kind: SignalKind::Err(msg.into())
        }
    }
}

/// Message Format
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SignalKind {
    Join,
    List,
    ShutDown,
    Ok, // 200
    Err(String)
}
