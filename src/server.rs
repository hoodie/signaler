//! Signaling Server
//!
//! Only on instance connecting many sessions
//!


use actix::prelude::*;
use log::{debug, info, trace, warn};
use rand::{self, rngs::ThreadRng, Rng};

use std::collections::{HashMap, HashSet};
use std::io;

use crate::protocol::public::*;
use crate::protocol::internal;


pub struct SignalingServer {
    // sessions: HashMap<usize, Recipient<SignalMessage>>,
    // rooms: HashMap<String, HashSet<usize>>,
    // rng: ThreadRng,
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
        // default room
        // let mut rooms = HashMap::new();
        // rooms.insert("Main".to_owned(), HashSet::new());

        SignalingServer {
            // sessions: HashMap::new(),
            // rooms: rooms,
            // rng: rand::thread_rng(),
        }
    }
}

impl Handler<internal::ListRooms> for SignalingServer {
    type Result = MessageResult<internal::ListRooms>;

    fn handle(&mut self, _: internal::ListRooms, _ctx: &mut Self::Context) -> Self::Result {
        // MessageResult(self.rooms.keys().cloned().collect())
        debug!("received listrequest from ...");
        MessageResult(
            vec![
                String::from("foo"),
                String::from("bar"),
            ]
        )
    }
}