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

        use room::command::RoomCommand;
        use SessionCommand::*;
        let session_id = self.session_id;

        match msg {
            Authenticate { .. } => log::warn!("unexpected authentication"),
            ListMyRooms => self.list_my_rooms(ctx),

            // to RoomManager
            ListRooms => self.list_rooms(ctx),
            Join { room } => self.join(&room, ctx),

            Leave { room } => self.send_to_room(RoomCommand::RemoveParticipant { session_id }, &room),
            ListParticipants { room } => self.send_to_room(RoomCommand::GetParticipants { session_id }, &room),

            ChatRoom { room, command } => {
                self.send_to_room(room::command::ChatRoomCommand { command, session_id }, &room)
            }

            ShutDown => System::current().stop(),
        }
    }

    /// send message to client
    fn send_message(&self, message: SessionMessage, _ctx: &mut Context<Self>) {
        if let Some(connection) = self.connection.as_ref().and_then(WeakAddr::upgrade) {
            log::trace!("send to connection {:?}", message);
            if let Err(error) = connection.try_send(crate::socket_connection::command::SessionMessage(message)) {
                log::error!("failed to send message to socket_connection {}", error)
            }
        } else {
            log::warn!("have no connection to send to");
        }
    }

    fn room_addr(&self, room_id: &str) -> Option<Addr<DefaultRoom>> {
        self.rooms.get(room_id).and_then(|room| room.upgrade())
    }

    // command to room
    fn send_to_room<C>(&self, command: C, room_id: &str)
    where
        C: Message + Send + 'static,
        <C as Message>::Result: Send,
        room::DefaultRoom: Handler<C>,
    {
        if let Some(addr) = self.room_addr(room_id) {
            if let Err(e) = addr.try_send(command) {
                log::error!("failed to send command to room {:?} {}", room_id, e);
            }
        } else {
            log::error!(
                "room: {:?} was no longer reachable by client-session {:?}, removing",
                room_id,
                self.session_id
            );
        }
    }

    fn gc(&mut self, ctx: &mut Context<Self>) {
        if self.connection.as_ref().and_then(WeakAddr::upgrade).is_none() {
            log::info!("connection lost, closing session {}", self.session_id);
            ctx.stop();
        }
    }

    // command to room manager
    fn join(&self, room: &str, ctx: &mut Context<Self>) {
        if let Some(token) = self.token {
            let msg = room_manager::command::RoomManagerCommand::JoinRoom {
                room: room.into(),
                participant: RosterParticipant {
                    session_id: self.session_id,
                    addr: ctx.address().downgrade(),
                    profile: self.profile.clone(),
                },
                // return_addr: ctx.address().recipient(),
                token,
            };

            if let Err(error) = RoomManagerService::from_registry().try_send(msg) {
                log::error!("can't join room, no authentication token {}", error)
            }
        } else {
            log::warn!("can't join room, no authentication token")
        }
    }

    // command to room manager
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

    // self
    fn list_my_rooms(&self, ctx: &mut Context<Self>) {
        let rooms = self.rooms.keys().cloned().collect::<Vec<String>>();
        log::debug!("my list request answered: {:?}", rooms);
        self.send_message(SessionMessage::MyRoomList { rooms }, ctx);
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

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        log::debug!("ClientSession stopped: {}", self.session_id);
    }
}

impl fmt::Debug for ClientSession {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ClientSession {{ {} }}", self.session_id)
    }
}
