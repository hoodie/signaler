use std::{collections::HashMap, time::Instant};

use hannibal::{Context, Service, WeakAddr};
use signaler_protocol as protocol;
use signaler_protocol::RoomId;
use tracing::log;
use uuid::Uuid;

use crate::room::command::ChatRoomCommand;
use crate::{
    room::{participant::RoomParticipant, Room},
    room_manager::{self, RoomManager},
};

use self::message::FromSession;

mod actor;
pub mod command;
pub mod message;

pub type SessionId = Uuid;

pub struct Session {
    pub session_id: SessionId,
    pub connection: Option<hannibal::Sender<message::FromSession>>,
    pub last_seen_connected: Instant,
    pub rooms: HashMap<RoomId, WeakAddr<Room>>,
}

impl std::fmt::Debug for Session {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Session")
            .field("session_id", &self.session_id)
            .field("connection is some?", &self.connection.is_some())
            .finish()
    }
}

impl Default for Session {
    fn default() -> Self {
        Session {
            session_id: Uuid::new_v4(),
            connection: None,
            last_seen_connected: Instant::now(),
            rooms: Default::default(),
        }
    }
}

impl Session {
    pub fn with_connection(connection: hannibal::Sender<message::FromSession>) -> Self {
        Session {
            connection: Some(connection),
            ..Default::default()
        }
    }

    pub async fn dispatch_command(&mut self, cmd: protocol::SessionCommand, ctx: &mut Context<Self>) {
        log::trace!("dispatching {cmd:#?}");
        match cmd {
            protocol::SessionCommand::Join { room } => self.join(room, ctx).await,
            protocol::SessionCommand::ChatRoom { room, command } => self.send_to_room(
                room,
                ChatRoomCommand {
                    command,
                    session_id: self.session_id,
                },
            ),
            protocol::SessionCommand::ListRooms => todo!(),
            protocol::SessionCommand::ListMyRooms => todo!(),
            protocol::SessionCommand::ShutDown => todo!(),
            protocol::SessionCommand::Authenticate { .. } => todo!(),
        }
    }

    pub async fn join(&self, room_id: RoomId, ctx: &mut Context<Self>) {
        log::debug!("join {room_id}");
        let msg = room_manager::Command::JoinRoom {
            room_id,
            participant: RoomParticipant {
                session_id: self.session_id,
                addr: ctx.address().downgrade(),
                profile: self.session_id.to_string(), // self.profile.clone(),
            },
            // return_addr: ctx.address().recipient(),
        };

        let rm = RoomManager::from_registry().await.unwrap();
        if let Err(error) = rm.send(msg) {
            log::error!("can't join room {error}")
        }
    }

    pub fn send_to_connection(&self, message: FromSession) {
        if let Some(ref connection) = self.connection {
            if connection.can_upgrade() {
                if let Err(e) = connection.send(message) {
                    log::warn!("failed to send to connection {}", e);
                }
            } else {
                log::warn!("connection can't upgrade");
            }
        } else {
            log::warn!("have no connection");
        }
    }

    pub fn send_to_room<C>(&self, room_id: RoomId, command: C)
    where
        C: hannibal::Message<Result = ()> + Send + 'static,
        crate::room::Room: hannibal::Handler<C>,
    {
        if let Some(room) = self.rooms.get(&room_id).and_then(WeakAddr::upgrade) {
            room.send(command).unwrap();
        }
    }
}

/// garbage collection
impl Session {
    fn gc(&mut self, ctx: &mut Context<Self>) {
        // log::trace!("gc");
        if let Some(can_upgrade) = self.connection.as_ref().map(|c| c.can_upgrade()) {
            if !can_upgrade {
                log::trace!("connection is gone");
                self.connection = None
            } else {
                // I'm still alive, updating timestamp
                self.last_seen_connected = Instant::now();
            }
        } else {
            let secs_since_disconnect = (Instant::now() - self.last_seen_connected).as_secs();
            log::trace!("session without connection {}s", secs_since_disconnect);
            if secs_since_disconnect > 10 {
                log::debug!(
                    "session without connection for more than {}s, stopping session",
                    secs_since_disconnect
                );
                ctx.stop(None);
            }
        }
    }
}
