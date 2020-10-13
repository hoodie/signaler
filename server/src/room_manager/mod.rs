//! RoomManager Actor etc

use actix::{prelude::*, WeakAddr};

use std::collections::HashMap;

use crate::room::{
    command::AddParticipant, message::RoomToSession, participant::RosterParticipant, DefaultRoom, RoomId,
};

pub mod command;

/// Hands out Addresses to `Room`s  and creates them if necessary.
#[derive(Default)]
pub struct RoomManagerService {
    pub rooms: HashMap<RoomId, Addr<DefaultRoom>>,
}

impl RoomManagerService {
    fn join_room(&mut self, name: &str, participant: RosterParticipant) {
        if let Some(room) = self.rooms.get(name) {
            log::trace!("found room {:?}, just join", name);
            // TODO: AWAOT!
            room.try_send(AddParticipant { participant }).unwrap();
        } else {
            let room = self.create_room(name);
            log::trace!(
                "no room found {:?}, create and then join {:#?}",
                name,
                self.list_rooms()
            );
            room.upgrade()
                .unwrap()
                .try_send(AddParticipant { participant })
                .unwrap();
        }
    }

    fn send_decline(&mut self, room_id: &str, participant: RosterParticipant) {
        participant
            .addr
            .upgrade()
            .unwrap()
            .try_send(RoomToSession::JoinDeclined { room: room_id.into() })
            .unwrap();
    }

    fn create_room(&mut self, name: &str) -> WeakAddr<DefaultRoom> {
        log::trace!("create room: {:?}", name);
        let room = DefaultRoom::new(name.into()).start();
        let weak_room = room.downgrade();
        self.rooms.insert(name.into(), room);
        weak_room
    }

    fn create_permanent_room(&mut self, name: &str) -> WeakAddr<DefaultRoom> {
        log::trace!("create permanent room: {:?}", name);
        let room = DefaultRoom::permanent(name.into()).start();
        let weak_room = room.downgrade();
        self.rooms.insert(name.into(), room);
        weak_room
    }

    fn list_rooms(&self) -> Vec<String> {
        self.rooms.keys().cloned().collect()
    }

    fn close_room(&mut self, room_id: RoomId) -> bool {
        self.rooms.remove(&room_id).is_some()
    }
}

impl Actor for RoomManagerService {
    type Context = Context<Self>;
    fn started(&mut self, _ctx: &mut Self::Context) {
        log::debug!("RoomManager started");
        self.create_permanent_room("default");
    }
}

impl SystemService for RoomManagerService {}
impl Supervised for RoomManagerService {}
