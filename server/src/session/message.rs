use hannibal::WeakAddr;
use signaler_protocol::SessionMessage;

use super::Session;

#[hannibal::message]
#[derive(Debug)]
pub enum FromSession {
    SessionMessage(SessionMessage),
    SessionAssociated { session: WeakAddr<Session> },
}

impl From<SessionMessage> for FromSession {
    fn from(inner: SessionMessage) -> Self {
        Self::SessionMessage(inner)
    }
}
