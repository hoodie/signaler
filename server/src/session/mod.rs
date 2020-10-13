//! # Signaling Session
//!
//! One session per participant

// TODO: how to timeout sessions?

use actix::{prelude::*, WeakAddr};
use uuid::Uuid;

use std::{collections::HashMap, fmt, time::Duration};

use crate::{
    presence::command::AuthToken,
    room::{self, participant::RosterParticipant, DefaultRoom, RoomId},
    room_manager::{self, RoomManagerService},
    socket_connection::SocketConnection,
    user_management::UserProfile,
};
use signaler_protocol::*;

pub mod command;

pub type SessionId = Uuid;

/// Communicates back and forth between WebSocket and other Actors.
///
/// Talks to `Room`s, `PresenceService` and `RoomService`
pub struct ClientSession {
    connection: Option<WeakAddr<SocketConnection>>, // should be a weak Recipient
    session_id: SessionId,
    token: Option<AuthToken>,
    profile: Option<UserProfile>,
    rooms: HashMap<RoomId, WeakAddr<DefaultRoom>>,
}

impl ClientSession {
    pub fn from_token_and_profile(token: AuthToken, profile: UserProfile) -> Self {
        ClientSession {
            connection: None,
            token: Some(token),
            profile: Some(profile),
            ..Default::default()
        }
    }

    /// parses raw string and passes it to `dispatch_incoming_message` or replies with error
    /// react to client messages
    fn dispatch_incoming_message(&self, msg: SessionCommand, ctx: &mut Context<Self>) {
        log::trace!("received {:?}", msg);
        match msg {
            SessionCommand::Authenticate { .. } => log::warn!("unexpected authentication"),

            SessionCommand::ListRooms => self.list_rooms(ctx),

            SessionCommand::Join { room } => self.join(&room, ctx),

            SessionCommand::Leave { room } => self.leave_room(&room, ctx),

            SessionCommand::ListMyRooms => self.list_my_rooms(ctx),

            SessionCommand::ListParticipants { room } => self.request_room_update(&room, ctx),

            SessionCommand::Message { message, room } => self.forward_message(message, &room, ctx),

            SessionCommand::ShutDown => System::current().stop(),
        }
    }

    /// send message to client
    fn send_message(&self, message: SessionMessage, ctx: &mut Context<Self>) {
        if let Some(connection) = self.connection.as_ref().and_then(WeakAddr::upgrade) {
            log::debug!("send to connection {:?}", message);
            connection
                .send(crate::socket_connection::command::SessionMessage(message))
                .into_actor(self)
                .then(|_, _, _| fut::ready(()))
                .spawn(ctx);
        } else {
            log::warn!("have no connection to send to");
        }
    }

    fn gc(&mut self, ctx: &mut Context<Self>) {
        if self.connection.as_ref().and_then(WeakAddr::upgrade).is_none() {
            log::info!("connection lost, closing session {}", self.session_id);
            ctx.stop();
        }
    }

    fn list_rooms(&self, ctx: &mut Context<Self>) {
        let msg = room_manager::command::ListRooms;

        RoomManagerService::from_registry()
            .send(msg)
            .into_actor(self)
            .then(|rooms, slf, ctx| {
                log::debug!("list request answered: {:?}", rooms);
                match rooms {
                    Ok(rooms) => slf.send_message(SessionMessage::RoomList { rooms }, ctx),
                    Err(error) => slf.send_message(SessionMessage::err(format!("{:?}", error)), ctx),
                }
                fut::ready(())
            })
            .spawn(ctx);
    }

    fn list_my_rooms(&self, ctx: &mut Context<Self>) {
        let rooms = self.rooms.keys().cloned().collect::<Vec<String>>();
        log::debug!("my list request answered: {:?}", rooms);
        self.send_message(SessionMessage::MyRoomList { rooms }, ctx);
    }

    fn join(&self, room: &str, ctx: &mut Context<Self>) {
        if let Some(token) = self.token {
            let msg = room_manager::command::JoinRoom {
                room: room.into(),
                participant: RosterParticipant {
                    session_id: self.session_id,
                    addr: ctx.address().downgrade(),
                    profile: self.profile.clone(),
                },
                // return_addr: ctx.address().recipient(),
                token,
            };

            RoomManagerService::from_registry()
                .send(msg)
                .into_actor(self)
                .then(|_, _, _| {
                    log::debug!("join request forwarded to room successfully");
                    fut::ready(())
                })
                .spawn(ctx);
        } else {
            log::warn!("can't join room, no authentication token")
        }
    }

    fn leave_room(&self, room_id: &str, ctx: &mut Context<Self>) {
        if let Some(addr) = self.room_addr(room_id) {
            addr.send(room::command::RemoveParticipant {
                session_id: self.session_id,
            })
            .into_actor(self)
            .then(|_, _, _| {
                log::debug!("leave request forwarded to room successfully");
                fut::ready(())
            })
            .spawn(ctx);
        } else {
            log::error!("no such room {:?}", room_id);
        }
    }

    fn leave_all_rooms(&mut self, _ctx: &mut Context<Self>) {}

    fn room_addr(&self, room_id: &str) -> Option<Addr<DefaultRoom>> {
        let addr = self.rooms.get(room_id).and_then(|room| room.upgrade());
        if addr.is_none() {
            log::warn!(
                "room: {:?} was no longer reachable by client-session {:?}, removing",
                room_id,
                self.session_id
            );
            // self.rooms.remove(room_id); // TODO: do at Interval
        }
        addr
    }

    fn forward_message(&self, content: String, room_id: &str, ctx: &mut Context<Self>) {
        let full_name = if let Some(ref profile) = self.profile {
            profile.full_name.as_ref()
        } else {
            "unnamed"
        };

        let msg = room::command::Forward {
            message: ChatMessage::new(content, self.session_id, full_name),
            sender: self.session_id,
        };

        if let Some(addr) = self.room_addr(room_id) {
            addr.send(msg)
                .into_actor(self)
                .then(|resp, slf, ctx| {
                    log::debug!("message forwarded -> {:?}", resp);
                    match resp {
                        Ok(Err(message)) => {
                            log::debug!("message rejected {:?}", message);
                            slf.send_message(SessionMessage::Error { message }, ctx)
                        }
                        Ok(mr) => {
                            log::debug!("ok after all -> {:?}", mr);
                        }
                        Err(error) => slf.send_message(
                            SessionMessage::Error {
                                message: error.to_string(),
                            },
                            ctx,
                        ),
                    }
                    fut::ready(())
                })
                .spawn(ctx);
        } else {
            log::error!("no such room {:?}", room_id);
        }
    }

    fn request_room_update(&self, room_id: &str, ctx: &mut Context<Self>) {
        let room_name = room_id.to_owned();
        if let Some(addr) = self.room_addr(room_id) {
            addr.send(room::command::RoomUpdate)
                .into_actor(self)
                .then(|resp, slf, ctx| {
                    log::debug!("received response for ListParticipants request");
                    match resp {
                        Ok(participants) => slf.send_message(
                            SessionMessage::RoomParticipants {
                                room: room_name,
                                participants,
                            },
                            ctx,
                        ),
                        Err(error) => slf.send_message(
                            SessionMessage::Error {
                                message: error.to_string(),
                            },
                            ctx,
                        ),
                    }
                    fut::ready(())
                })
                .spawn(ctx);
        }
    }
}

impl Default for ClientSession {
    fn default() -> Self {
        Self {
            connection: None,
            session_id: Uuid::new_v4(),
            token: None,
            profile: None,
            rooms: Default::default(),
        }
    }
}

impl Actor for ClientSession {
    type Context = Context<Self>;
    fn started(&mut self, ctx: &mut Self::Context) {
        log::info!("ClientSession started {:?}", self.session_id);
        self.send_message(
            SessionMessage::Welcome {
                session: SessionDescription {
                    session_id: self.session_id,
                },
            },
            ctx,
        );
        ctx.run_interval(Duration::from_millis(5_000), Self::gc);
    }

    fn stopping(&mut self, ctx: &mut Self::Context) -> Running {
        self.leave_all_rooms(ctx);

        Running::Stop
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        log::debug!("ClientSession stopped: {}", self.session_id);
    }
}

impl fmt::Debug for ClientSession {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ClientSession {{ {} }}", self.session_id)
    }
}
