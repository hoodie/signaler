// extern crate actix;

// use actix::prelude::*;
use rand::{self, rngs::ThreadRng, Rng};
use std::collections::{HashMap, HashSet};

use crate::SignalingMessage;


pub struct SignalingServer {
    sessions: HashMap<usize, Recipient<SignalingMessage>>,
    rooms: HashMap<String, HashSet<usize>>,
    rng: ThreadRng,
}

impl Actor for SignalingServer {
    type Context = Context<Self>;
}

impl Default for SignalingServer {
    fn default() -> ChatServer {
        // default room
        let mut rooms = HashMap::new();
        rooms.insert("Main".to_owned(), HashSet::new());

        SignalingServer {
            sessions: HashMap::new(),
            rooms: rooms,
            rng: rand::thread_rng(),
        }
    }
}