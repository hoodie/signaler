//! Signaling Server
//!
//! Only on instance connecting many sessions
//!


use actix::prelude::*;
use log::{debug, info, trace, warn};
use uuid::Uuid;

use std::collections::{HashMap, HashSet};
use std::io;

use crate::protocol::public::*;
use crate::protocol::internal;

pub struct SignalingServer {
    sessions: HashMap<Uuid, Recipient<internal::ServerToSession>>,
    rooms: HashMap<String, HashSet<Uuid>>,
}

impl SystemService for SignalingServer {}
impl Supervised for SignalingServer {}

impl Actor for SignalingServer {
    type Context = Context<Self>;
    fn started(&mut self, _ctx: &mut Self::Context) {
        info!("signaling server started")
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        info!("signaling stopped")
    }
}

impl Default for SignalingServer {
    fn default() -> SignalingServer {

        SignalingServer {
            sessions: HashMap::new(),
            rooms: HashMap::new(),
        }
    }
}

impl Handler<internal::Ping> for SignalingServer {
    type Result = ();

    fn handle(&mut self, _: internal::Ping, _ctx: &mut Self::Context) -> Self::Result {
        info!("received ping");
    }
}

impl Handler<internal::ListRooms> for SignalingServer {
    type Result = MessageResult<internal::ListRooms>;

    fn handle(&mut self, _: internal::ListRooms, _ctx: &mut Self::Context) -> Self::Result {
        // MessageResult(self.rooms.keys().cloned().collect())
        info!("received listrequest from ...");
        MessageResult(
            self.rooms.keys().cloned().collect()
        )
    }
}


impl Handler<internal::JoinRoom> for SignalingServer {
    type Result = ();

    fn handle(&mut self, join: internal::JoinRoom, _ctx: &mut Self::Context) -> Self::Result {
        self.sessions.insert(join.uuid, join.addr);
        let participants = self.rooms
            .entry(join.room.clone())
            .or_insert(Default::default())
            .insert(join.uuid);

        debug!("rooms: {:?}, paricipants of {:?}: {:#?}", self.rooms, join.room, participants);
    }
}

