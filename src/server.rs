//! Signaling Server
//!
//! Only on instance connecting many sessions
//!


use actix::prelude::*;
use log::{debug, info, trace, warn, error};
use uuid::Uuid;

use std::collections::{HashMap, HashSet};
use std::io;

use crate::protocol::public::*;
use crate::protocol::internal::{self, RoomId};

pub struct SignalingServer {
    sessions: HashMap<Uuid, Recipient<internal::ServerToSession>>,
    rooms: HashMap<RoomId, HashSet<Uuid>>,
}

impl SignalingServer {
    pub fn debug_rooms(&self) {
        let room_counts = self.rooms.iter().map(|(room, uuids)| (room, uuids.len())).collect::<HashMap<_,_>>();
        debug!("room participants {:#?}", room_counts);
    }
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

//// message Handlers 

impl Handler<internal::Ping> for SignalingServer {
    type Result = ();

    fn handle(&mut self, _: internal::Ping, _ctx: &mut Self::Context) -> Self::Result {
        info!("received ping");
    }
}

impl Handler<internal::ServerToSession> for SignalingServer {
    type Result = ();

    fn handle(&mut self, fwd_msg: internal::ServerToSession, ctx: &mut Self::Context) -> Self::Result {
        use internal::ServerToSession::*;

        info!("server received ServerToSession::{:?}", fwd_msg);
        match fwd_msg {
            Forward(chat_msg, roomid) => {
                if let Some(participants) = self.rooms.get(&roomid) {
                    for participant in participants {
                    // if let Some(participant) = participants.iter().nth(0) {
                        let session = self.sessions.get(&participant).unwrap();

                        trace!("forwarding message to {:#?}", participant);

                        session
                            .send(ChatMessage(chat_msg.clone()))
                            .into_actor(self)
                            .then(|_,_,_|{
                                trace!("chatmessages passed on");
                                fut::ok(())
                            })
                            .spawn(ctx);
                    }
                }
            },
            ChatMessage(msg) => error!("the server should not receive chat messages, use Forward\n{:?}", msg),
        }
    }
}

impl Handler<internal::ListRooms> for SignalingServer {
    type Result = MessageResult<internal::ListRooms>;
    fn handle(&mut self, _: internal::ListRooms, _ctx: &mut Self::Context) -> Self::Result {
        info!("received listrequest from ...");
        MessageResult(
            self.rooms.keys().cloned().collect()
        )
    }
}

impl Handler<internal::ListMyRooms> for SignalingServer {
    type Result = MessageResult<internal::ListMyRooms>;

    fn handle(&mut self, me: internal::ListMyRooms, _ctx: &mut Self::Context) -> Self::Result {
        info!("received listrequest from ...");
        MessageResult(
            self.rooms
                .iter()
                .filter(|(_room, participants)| participants.iter().any(|&uuid| uuid == me.uuid))
                .map(|(room, _)| room)
                .cloned()
                .collect()
        )
    }
}

impl Handler<internal::JoinRoom> for SignalingServer {
    type Result = MessageResult<internal::JoinRoom>;

    fn handle(&mut self, join: internal::JoinRoom, _ctx: &mut Self::Context) -> Self::Result {
        self.sessions.insert(join.uuid, join.addr);
        if join.room.len() == 0 {
            error!("listname must'n be empty");
            return MessageResult(Err("listname must'n be empty".into()));
        }

        let participants = self.rooms
            .entry(join.room.clone())
            .or_insert(Default::default())
            .insert(join.uuid);

        debug!("rooms: {:#?}, paricipants of {:?}: {:#?}", self.rooms, join.room, participants);
        self.debug_rooms();
        MessageResult(Ok(()))
    }
}

