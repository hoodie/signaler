use actix::{prelude::*, utils::IntervalFunc, WeakAddr};
use actix_web_actors::ws::{self, WebsocketContext};
use uuid::Uuid;

use std::time::Duration;

use crate::{session::ClientSession, session_manager::SessionManagerService};
use signaler_protocol::*;

pub(crate) type DynMessageHandler = dyn (Fn(&SocketConnection, &str, &mut WebsocketContext<SocketConnection>));

pub mod command;

/// Terminates a WebSocket connection and dispatches Messages
pub struct SocketConnection {
    connection_id: Uuid,
    session: Option<WeakAddr<ClientSession>>,
    message_handler: Box<DynMessageHandler>,
}

impl SocketConnection {
    /// gets all the messages and dispatches to a specific handler, either `handle_connection_message`
    fn handle_incoming_message(&self, raw_msg: &str, ctx: &mut WebsocketContext<Self>) {
        let handler = &self.message_handler;
        handler(self, raw_msg, ctx);
    }

    /// parses raw string and passes it to `dispatch_incoming_message` or replies with error
    fn handle_connection_message(&self, raw_msg: &str, ctx: &mut WebsocketContext<Self>) {
        log::debug!("handle connection message: {:?}", raw_msg);
        let parsed: Result<command::ConnectionCommand, _> = serde_json::from_str(raw_msg);
        if let Ok(msg) = parsed {
            log::trace!("parsed ok\n{}\n{:?}", raw_msg, msg);
            match msg {
                command::ConnectionCommand::Authenticate { credentials } => self.associate_session(credentials, ctx),
            }
        } else {
            log::warn!("cannot parse: {}", raw_msg);
        }
    }

    fn handle_session_message(&self, raw_msg: &str, ctx: &mut WebsocketContext<Self>) {
        log::info!("handle session message: {:?}", raw_msg);
        let parsed: Result<SessionCommand, _> = serde_json::from_str(raw_msg);
        if let Ok(msg) = parsed {
            log::trace!("parsed ok\n{}\n{:?}", raw_msg, msg);
            if let Some(ref session) = self.session.as_ref().and_then(|a| a.upgrade()) {
                let msg = crate::session::command::SessionCommand(msg);
                session
                    .send(msg)
                    .into_actor(self) // .actfuture()
                    .then(|_, _, _| fut::ready(()))
                    .spawn(ctx);
            }
        } else {
            log::warn!("cannot parse: {}", raw_msg);
            let command = SessionCommand::ChatRoom {
                room: "room_id".into(),
                command: ChatRoomCommand::Leave,
            };
            log::trace!("please try {}", serde_json::to_string_pretty(&command).unwrap());
        }
    }

    fn associate_session(&self, credentials: Credentials, ctx: &mut WebsocketContext<Self>) {
        let connection = ctx.address().downgrade().recipient();

        SessionManagerService::from_registry()
            .try_send(crate::session_manager::command::GetSession {
                credentials,
                connection,
            })
            .unwrap();
    }

    /// send message to client
    fn send_message(message: SessionMessage, ctx: &mut WebsocketContext<Self>) {
        ctx.text(message.into_json())
    }

    fn send_ping(&mut self, ctx: &mut WebsocketContext<Self>) {
        let ping_msg = self.connection_id.to_string();
        ctx.ping(ping_msg.as_bytes());
    }
}

impl Default for SocketConnection {
    fn default() -> Self {
        Self {
            connection_id: Uuid::new_v4(),
            session: None,
            message_handler: Box::new(Self::handle_connection_message),
        }
    }
}

impl Actor for SocketConnection {
    type Context = WebsocketContext<Self>;
    fn started(&mut self, ctx: &mut Self::Context) {
        log::trace!("ClientConnection started {:?}", self.connection_id);

        Self::send_message(
            SessionMessage::Welcome {
                session: SessionDescription {
                    session_id: self.connection_id,
                },
            },
            ctx,
        );
        IntervalFunc::new(Duration::from_millis(5_000), Self::send_ping)
            .finish()
            .spawn(ctx);
    }

    fn stopping(&mut self, _ctx: &mut Self::Context) -> Running {
        Running::Stop
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        log::trace!("ClientConnection stopped: {}", self.connection_id);
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for SocketConnection {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                log::warn!("PING -> PONG");
                ctx.pong(&msg)
            }
            Ok(ws::Message::Pong(_msg)) => {
                // log::trace!("received PONG {:?}", msg);
            }
            Ok(ws::Message::Text(text)) => {
                self.handle_incoming_message(&text, ctx);
            }
            Ok(ws::Message::Close(reason)) => {
                log::info!("websocket was closed {:?}", reason);
                ctx.stop();
            }
            Err(e) => {
                log::warn!("websocket was closed because of error {:?}", e);
                ctx.stop();
            }
            _ => (), // Pong, Nop, Binary
        }
    }
}
