//! RoomManager Actor etc

use actix::{prelude::*, WeakAddr};

use std::collections::HashMap;

use crate::room::{self, message::RoomToSession, participant::RosterParticipant, DefaultRoom, RoomId};

pub mod command;

/// Hands out Addresses to `Room`s  and creates them if necessary.
#[derive(Default)]
pub struct RoomManagerService {
    pub rooms: HashMap<RoomId, Addr<DefaultRoom>>,
}

impl RoomManagerService {
    fn join_room(&mut self, room: &RoomId, participant: RosterParticipant) {
        if let Some(room) = self.rooms.get(room).cloned().or_else(|| {
            let room = self.create_room(room).upgrade();
            log::trace!("no room found {:?}, creating", room);
            room
        }) {
            if let Err(error) = room.try_send(room::Command::AddParticipant { participant }) {
                log::error!("failed to add participant to room {}", error)
            }
        }
    }

    fn send_decline(&mut self, room_id: &str, participant: RosterParticipant) {
        if let Some(participant) = participant.addr.upgrade() {
            if let Err(error) = participant.try_send(RoomToSession::JoinDeclined { room: room_id.into() }) {
                log::error!("failed to send decline to particiapnt {:?}{}", participant, error);
            }
        } else {
            log::error!("participant was no longer reachable from RoomManager {:?}", participant);
        }
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
        self.rooms.keys().map(ToString::to_string).collect()
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
