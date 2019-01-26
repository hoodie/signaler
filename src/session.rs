//! # Signaling Session
//!
//! One session per participant

// TODO: how to timeout sessions?

use ::actix::prelude::*;
use actix_web::ws::{self, WebsocketContext};
use log::*;
use uuid::Uuid;
use serde_derive::Serialize;

use crate::protocol::*;
use crate::server::{self, SignalingServer};

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
            debug!("parsed ok\n{}\n{:#?}", raw_msg, msg);
            self.dispatch_incoming_message(msg, ctx)

        } else {
            warn!("cannot parse: {}", raw_msg);
        }
    }

    /// react to client messages
    fn dispatch_incoming_message(&self, msg: SessionCommand, ctx: &mut WebsocketContext<Self>) {
        match msg.kind {
            SessionCommandKind::ListRooms => {
                debug!("received ListRooms signal");
                self.list_rooms(ctx);
            },
            SessionCommandKind::Join{ room } => {
                debug!("requesting to Join {}", room);
                self.join(&room, ctx);
            },
            SessionCommandKind::ListMyRooms => {
                debug!("received ListMyRooms signal");
                self.list_my_rooms(ctx);
            },
            SessionCommandKind::Message{ message, room } => {
                debug!("received message to forward");
                self.forward_message(message, &room, ctx);
            },
            SessionCommandKind::ShutDown => {
                debug!("received shut down signal");
                System::current().stop();
            },
            _ => {}
        }
        Self::send_message(SessionMessageKind::Ok, ctx);
    }

    /// send message to client
    fn send_message(kind: SessionMessageKind, ctx: &mut WebsocketContext<Self>) {
        ctx.text(SessionMessage::from(kind).to_json())
    }

    fn list_rooms(&self, ctx: &mut WebsocketContext<Self>) {
        let msg = server::command::ListRooms;

        SignalingServer::from_registry()
            .send(msg)
            .into_actor(self)
            .then(|rooms, _, ctx| {
                debug!("list request answered: {:?}", rooms);
                match rooms {
                    Ok(rooms) => Self::send_message(SessionMessageKind::RoomList(rooms), ctx),
                    Err(error) => ctx.text(&SessionMessage::err(format!("{:#?}", error)).to_json())
                }
                fut::ok(())
            })
            .spawn(ctx);
    }

    fn list_my_rooms(&self, ctx: &mut WebsocketContext<Self>) {
        let msg = server::command::ListMyRooms {
            uuid: self.uuid
        };

        SignalingServer::from_registry()
            .send(msg)
            .into_actor(self)
            .then(|rooms, _, ctx| {
                debug!("my list request answered: {:?}", rooms);
                match rooms {
                    Ok(rooms) => Self::send_message(SessionMessageKind::RoomList(rooms), ctx),
                    Err(error) => ctx.text(&SessionMessage::err(format!("{:#?}", error)).to_json())
                }
                fut::ok(())
            })
            .spawn(ctx);
    }

    fn join(&self, room: &str, ctx: &mut WebsocketContext<Self>) {
        let msg = server::command::JoinRoom {
            room: room.into(),
            uuid: self.uuid,
            addr: ctx.address().recipient()
        };

        SignalingServer::from_registry()
            .send(msg)
            .into_actor(self)
            .then(|joined, s, ctx| {
                debug!("join request answered: {:?}", joined);
                match joined {
                    Ok(list) => {
                        ctx.text(&SessionMessage::any(list).to_json());
                        s.list_my_rooms(ctx);
                    },
                    Err(error) => ctx.text(&SessionMessage::err(format!("{:#?}", error)).to_json())
                }
                fut::ok(())
            })
            .spawn(ctx);
    }

    fn forward_message(&self, message: ChatMessage, room: &server::RoomId, ctx: &mut WebsocketContext<Self>) {
        let msg = server::command::Forward{message, room: room.clone()};
        SignalingServer::from_registry()
            .send(msg)
            .into_actor(self)
            .then(|_, _, _ctx| {
                debug!("message forwared");
                fut::ok(())
            })
            .spawn(ctx);
    }
}

// TODO: imple drop to delete sessions | add keepalive

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

impl Handler<server::message::ServerToSession> for ClientSession {
    type Result = ();

    fn handle(&mut self, s2s_msg: server::message::ServerToSession, ctx: &mut Self::Context) -> Self::Result {
        info!("received message from server {:?}", s2s_msg);
        match s2s_msg {
            server::message::ServerToSession::ChatMessage{ message, room } => {
                Self::send_message(SessionMessageKind::Message{message, room}, ctx)
            },
        }
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

