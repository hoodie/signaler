use signaler_protocol::SessionMessage;

#[hannibal::message]
#[derive(Debug)]
pub struct FromSession(SessionMessage);

impl From<SessionMessage> for FromSession {
    fn from(inner: SessionMessage) -> Self {
        Self(inner)
    }
}

impl From<FromSession> for SessionMessage {
    fn from(val: FromSession) -> Self {
        val.0
    }
}
