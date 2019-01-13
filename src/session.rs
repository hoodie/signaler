//! # Signaling Session
//!
//! One session per participant

// TODO: how to timeout sessions?

use ::actix::prelude::*;
use actix_web::ws::{self, WebsocketContext};
use log::{debug, info, trace, warn};
use uuid::Uuid;
use serde_derive::Serialize;

use crate::protocol::internal;
use crate::protocol::public::*;
use crate::server::SignalingServer;

#[derive(Clone, Debug, Serialize)]
pub struct ClientSession {
    uuid: Uuid,
}

impl ClientSession {

    /// parses raw string and passes it to `dispatch_incoming_message` or replies with error
    fn handle_incoming_message(&self, raw_msg: &str, ctx: &mut WebsocketContext<Self>) {
        // trace!("handle message: {:?}", raw_msg);
        let parsed: Result<SessionCommand, _> = serde_json::from_str(raw_msg);
        if let Ok(msg) = parsed {
            debug!("parsed ok\n{:#?}", msg);
            self.dispatch_incoming_message(msg, ctx)

        } else {
            warn!("cannot parse: {:#?}", raw_msg);
            trace!("allowed: {}", Self::allowed_messages());
        }
    }

    fn dispatch_incoming_message(&self, msg: SessionCommand, ctx: &mut WebsocketContext<Self>) {
        match msg.kind {
            SessionCommandKind::Join{room} => {
                debug!("requesting to join {}", room);
                self.join(&room, ctx);
            },
            SessionCommandKind::ShutDown => {
                debug!("received shut down signal");
                System::current().stop();
            },
            SessionCommandKind::ListRooms => {
                debug!("received list signal");
            },
            _ => {}
        }
        Self::send_message(SessionMessageKind::Ok, ctx);
    }

    fn join(&self, room: &str, ctx: &mut WebsocketContext<Self>) {
        let msg = internal::JoinRoom {
            room: room.into(),
            uuid: self.uuid,
            addr: ctx.address().recipient()
        };

        SignalingServer::from_registry()
            .send(msg)
            .into_actor(self)
            .then(|joined, _, ctx| {
                debug!("join request answered: {:?}", joined);
                match joined {
                    Ok(list) => ctx.text(&SessionMessage::any(list).to_json()),
                    Err(error) => ctx.text(&SessionMessage::err(format!("{:#?}", error)).to_json())
                }
                fut::ok(())
            })
            .spawn(ctx);
    }

    fn send_message(kind: SessionMessageKind, ctx: &mut WebsocketContext<Self>) {
        ctx.text(SessionMessage::from(kind).to_json())
    }

    fn allowed_messages() -> String {
        format!("{}\n{}",
                serde_json::to_string_pretty(&SessionCommand::join()).unwrap(),
                serde_json::to_string_pretty(&SessionCommand::list()).unwrap()
                )
    }

}

impl Default for ClientSession {
    fn default() -> Self {
        Self {
            uuid: Uuid::new_v4()
        }
    }
}

impl Actor for ClientSession {
    type Context = WebsocketContext<Self>;
    fn started(&mut self, ctx: &mut Self::Context) {
        info!("ClientSession started {:?}", self);
        ClientSession::send_message(SessionMessageKind::Welcome{ session: self.clone() }, ctx); 
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        debug!("ClientSsession stopped")
    }
}

impl Handler<internal::ServerToSession> for ClientSession {
    type Result = ();

    fn handle(&mut self, _: internal::ServerToSession, _ctx: &mut Self::Context) -> Self::Result {
        info!("received message from server");
    }
}

impl StreamHandler<ws::Message, ws::ProtocolError> for ClientSession {
    fn handle(&mut self, msg: ws::Message, ctx: &mut Self::Context) {
        match msg {
            ws::Message::Ping(msg) => ctx.pong(&msg),
            ws::Message::Text(text) => {
                self.handle_incoming_message(&text, ctx);
            },
            _ => (),
        }
    }
}

