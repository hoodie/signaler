//! # Client Connection
//!
//! Terminates WebSocket and forwards to Session

#[allow(unused_imports)]
use actix::{prelude::*, utils::IntervalFunc, WeakAddr};
use actix_web_actors::ws::{self, WebsocketContext};

#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};
use uuid::Uuid;

#[allow(unused_imports)]
use std::{collections::HashMap, fmt, time::Duration};

use signaler_protocol::*;

#[allow(unused_imports)]
use crate::{
    presence::{
        command::{AuthToken, AuthenticationRequest},
        message::AuthResponse,
        Credentials, SimplePresenceService,
    },
    room::{self, message::RoomToSession, participant::RosterParticipant, DefaultRoom, RoomId},
    room_manager::{self, RoomManagerService},
    user_management::UserProfile,
};


pub struct ClientConnection {
    connection_id: Uuid,
    // session: Option<WeakAddr<Session>>
}

impl ClientConnection {
    fn handle_incoming_message(&self, raw_msg: &str, ctx: &mut WebsocketContext<Self>) {
        let parsed: Result<SessionCommand, _> = serde_json::from_str(raw_msg);
        if let Ok(msg) = parsed {
            trace!("parsed ok\n{}\n{:?}", raw_msg, msg);
        //TODO:  self.dispatch_incoming_message(msg, ctx)
        } else {
            warn!("cannot parse: {}", raw_msg);
            debug!("suggestions:\n{}", SessionCommand::suggestions())
        }
    }

    fn send_ping(&mut self, ctx: &mut WebsocketContext<Self>) {
        let ping_msg = self.connection_id.to_string();
        ctx.ping(ping_msg.as_bytes());
        trace!("sent     PING {:?}", ping_msg);
    }

    /// send message to client
    fn send_message(message: SessionMessage, ctx: &mut WebsocketContext<Self>) {
        ctx.text(message.into_json())
    }

    fn authenticate(&self, credentials: Credentials, ctx: &mut WebsocketContext<Self>) {
        trace!("session starts authentication process");
        let msg = AuthenticationRequest {
            credentials,
        };
        // SimplePresenceService::from_registry()
        //     .send(msg)
        //     .into_actor(self)
        //     .then(|profile, client_session, ctx| {
        //         debug!("userProfile {:?}", profile);
        //         match profile {
        //             Ok(Some(AuthResponse { token, profile })) => {
        //                 info!("authenticated {:?}", token);
        //                 client_session.token = Some(token);
        //                 client_session.profile = Some(profile.clone());
        //                 Self::send_message(SessionMessage::Authenticated, ctx);
        //                 Self::send_message(
        //                     SessionMessage::Profile {
        //                         profile: profile.into(),
        //                     },
        //                     ctx,
        //                 );
        //             }
        //             Ok(None) => Self::send_message(
        //                 SessionMessage::Error {
        //                     message: String::from("unable to login"),
        //                 },
        //                 ctx,
        //             ),
        //             Err(error) => ctx.text(SessionMessage::err(format!("{:?}", error)).into_json()),
        //         }
        //         fut::ready(())
        //     })
        //     .spawn(ctx);
    }
}

impl Default for ClientConnection {
    fn default() -> Self {
        Self {
            connection_id: Uuid::new_v4(),
        }
    }
}

impl SessionCommandDispatcher for ClientConnection {
    type Context = WebsocketContext<Self>;
    fn dispatch_command(&self, msg: SessionCommand, ctx: &mut Self::Context) {
        trace!("connection received {:?}", msg);
        match msg {
            SessionCommand::Authenticate { credentials } => self.authenticate(credentials, ctx),
            _ => {
                warn!("unhandled message, forward to Session {:?}", msg);   
                // TODO forward the rest to session
            }

            // SessionCommand::ListRooms => self.list_rooms(ctx),
            // SessionCommand::Join { room } => self.join(&room, ctx),
            // SessionCommand::Leave { room } => self.leave_room(&room, ctx),
            // SessionCommand::ListMyRooms => self.list_my_rooms(ctx),
            // SessionCommand::ListParticipants { room } => self.request_room_update(&room, ctx),
            // SessionCommand::Message { message, room } => self.forward_message(message, &room, ctx),
            // SessionCommand::ShutDown => System::current().stop(),
        }
    }
}

impl Actor for ClientConnection {
    type Context = WebsocketContext<Self>;
    fn started(&mut self, ctx: &mut Self::Context) {
        info!("ClientConnection opened {:?}", self.connection_id);
        // ClientSession::send_message(
        //     SessionMessage::Welcome {
        //         session: SessionDescription {
        //             session_id: self.session_id,
        //         },
        //     },
        //     ctx,
        // );
        IntervalFunc::new(Duration::from_millis(5_000), Self::send_ping)
            .finish()
            .spawn(ctx);
    }

    fn stopping(&mut self, ctx: &mut Self::Context) -> Running {
        // self.leave_all_rooms(ctx);

        Running::Stop
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        debug!("ClientConnection stopped: {}", self.connection_id);
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for ClientConnection {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                warn!("PING -> PONG");
                ctx.pong(&msg)
            }
            Ok(ws::Message::Pong(msg)) => {
                trace!("received PONG {:?}", msg);
            }
            Ok(ws::Message::Text(text)) => {
                self.handle_incoming_message(&text, ctx);
            }
            Ok(ws::Message::Close(reason)) => {
                info!("websocket was closed {:?}", reason);
                ctx.stop();
            }
            Err(e) => {
                warn!("websocket was closed because of error {:?}", e);
                ctx.stop();
            }
            _ => (), // Pong, Nop, Binary
        }
    }
}
