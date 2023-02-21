use std::collections::HashMap;

use hannibal::{Actor, Addr, Context, WeakAddr};
use prometheus::IntGauge;
use tracing::log;

use crate::room::{self, participant::RoomParticipant, Room, RoomId};

mod actor;
pub mod command;

pub use command::Command;

#[derive(Debug, Default)]
pub struct RoomManager {
    pub rooms: HashMap<RoomId, Addr<Room>>,
    open_rooms: Option<IntGauge>,
}

impl RoomManager {
    async fn join_room(&mut self, room: &RoomId, participant: RoomParticipant) {
        log::debug!("join {room} with {participant:?}");
        let existing_room = self.rooms.get(room).cloned();
        let new_room = if existing_room.is_none() {
            log::trace!("no room found {:?}, creating", existing_room);
            self.create_room(room).await.upgrade()
        } else {
            None
        };

        if let Some(room) = existing_room.or(new_room) {
            if let Err(error) = room.send(room::Command::AddParticipant { participant }) {
                log::error!("failed to add participant to room {}", error)
            }
        }
    }

    async fn create_room(&mut self, name: &str) -> WeakAddr<Room> {
        log::debug!("create room: {:?}", name);
        let room = Room::new(name.into()).start().await.unwrap();
        let weak_room = room.downgrade();
        self.rooms.insert(name.into(), room);
        if let Some(gauge) = self.open_rooms.as_ref() {
            gauge.inc();
            log::trace!("increasing rooms count {:?}", gauge.get());
        }
        weak_room
    }

    // fn list_rooms(&self) -> Vec<String> {
    //     self.rooms.keys().map(ToString::to_string).collect()
    // }

    // fn close_room(&mut self, room_id: RoomId) -> bool {
    //     self.rooms.remove(&room_id).is_some()
    // }
}

impl RoomManager {
    fn gc(&mut self, _ctx: &mut Context<Self>) {
        // log::trace!("gc");
        self.rooms.retain(|id, room| {
            if room.stopped() {
                log::trace!("room {} has stopped", id);
                if let Some(gauge) = self.open_rooms.as_ref() {
                    gauge.dec();
                    log::trace!("decreasing rooms count {:?}", gauge.get());
                }
                false
            } else {
                true
            }
        });
    }
}
