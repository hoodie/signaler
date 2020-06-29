use super::*;
use crate::session::ClientSession;
use serde::{Deserialize, Serialize};

/// Command sent to the server
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type")]
#[rustfmt::skip]
pub enum ConnectionCommand {
    /// Request Authentication Token
    Authenticate { credentials: Credentials },
}

#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct SessionConnected {
    pub session: WeakAddr<ClientSession>,
}

impl Handler<SessionConnected> for SocketConnection {
    type Result = ();
    fn handle(&mut self, SessionConnected { session }: SessionConnected, ctx: &mut Self::Context) -> Self::Result {
        log::warn!("connection switching to session mode");
        self.session.replace(session);
        self.message_handler = Box::new(Self::handle_session_message);
        if let Some(session) = self.session.as_ref().and_then(|s| s.upgrade()) {
            session
                .send(crate::session::command::RegisterConnection {
                    connection: ctx.address().downgrade(),
                })
                .into_actor(self) // .actfuture()
                .then(|_, _, _| fut::ready(()))
                .spawn(ctx);
            Self::send_message(signaler_protocol::SessionMessage::Authenticated, ctx);
        } else {
            log::warn!("received invalid session");
        }
    }
}

#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct SessionMessage(pub signaler_protocol::SessionMessage);

impl Handler<SessionMessage> for SocketConnection {
    type Result = ();
    fn handle(&mut self, msg: SessionMessage, ctx: &mut Self::Context) -> Self::Result {
        let SessionMessage(msg) = msg;
        log::debug!("SessionMessage received {:#?}", msg);
        ctx.text(serde_json::to_string(&msg).unwrap())
    }
}
