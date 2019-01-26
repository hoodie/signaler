//! Signaling Server
//!
//! Only on instance connecting many sessions
//!


use actix::prelude::*;
use log::*;
use uuid::Uuid;

use std::collections::{HashMap, HashSet};

pub struct SignalingServer {
    sessions: HashMap<Uuid, Recipient<message::ServerToSession>>,
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

//// messages 

use crate::protocol::ChatMessage;

pub type RoomId = String;

pub mod message {
    //! Backchannel for clients
    //! 
    //! Clients must be able to receive

    use actix::prelude::*;
    use super::{RoomId, ChatMessage};

    #[derive(Message, Debug)]
    #[rtype(result = "()")]
    pub enum ServerToSession {
        ChatMessage {
            room: RoomId,
            message: ChatMessage,
        }
    }
}

pub mod command {
    //! Messages the server can receive
    use actix::prelude::*;
    use log::*;
    use uuid::Uuid;
    use super::{RoomId, ChatMessage, SignalingServer};

    #[derive(Message)]
    #[rtype(result = "()")]
    pub struct Ping;

    impl Handler<Ping> for SignalingServer {
        type Result = ();

        fn handle(&mut self, _: Ping, _ctx: &mut Self::Context) -> Self::Result {
            info!("received ping");
        }
    }

    #[derive(Message)]
    #[rtype(result = "Result<(), String>")]
    pub struct JoinRoom {
        pub room: String,
        pub uuid: Uuid,
        pub addr: Recipient<super::message::ServerToSession>,
    }

    impl Handler<JoinRoom> for SignalingServer {
        type Result = MessageResult<JoinRoom>;

        fn handle(&mut self, join: JoinRoom, _ctx: &mut Self::Context) -> Self::Result {
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
    #[derive(Message)]
    #[rtype(result = "Vec<String>")]
    pub struct ListRooms;

    impl Handler<ListRooms> for SignalingServer {
        type Result = MessageResult<ListRooms>;
        fn handle(&mut self, _: ListRooms, _ctx: &mut Self::Context) -> Self::Result {
            info!("received listrequest from ...");
            MessageResult(
                self.rooms.keys().cloned().collect()
            )
        }
    }

    #[derive(Message)]
    #[rtype(result = "Vec<String>")]
    pub struct ListMyRooms {
        pub uuid: Uuid,
    }

    impl Handler<ListMyRooms> for SignalingServer {
        type Result = MessageResult<ListMyRooms>;

        fn handle(&mut self, me: ListMyRooms, _ctx: &mut Self::Context) -> Self::Result {
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

    /// TODO: check if sender is actually participantof that room
    #[derive(Debug, Message)]
    #[rtype(result = "()")]
    pub struct Forward {
        pub room: RoomId,
        pub message: ChatMessage,
    }

    impl Handler<Forward> for SignalingServer {
        type Result = ();

        fn handle(&mut self, fwd: Forward, ctx: &mut Self::Context) -> Self::Result {
            use super::message::ServerToSession;

            info!("server received ServerToSession::{:?}", fwd);
            if let Some(participants) = self.rooms.get(&fwd.room) {
                for participant in participants {
                    let session = self.sessions.get(&participant).unwrap();

                    trace!("forwarding message to {:#?}", participant);

                    session
                        .send(ServerToSession::ChatMessage {
                            message: fwd.message.clone(),
                            room: fwd.room.clone(),
                        })
                        .into_actor(self)
                        .then(|_,_,_|{
                            trace!("chatmessages passed on");
                            fut::ok(())
                        })
                        .spawn(ctx);
                }
            } else {
                warn!("no such room {:?}", fwd.room);
            }
        }
    }
  


    #[derive(Message)]
    #[rtype(result = "()")]
    pub struct LeaveRoom {
        pub room: String,
        pub uuid: Uuid,
        pub addr: Recipient<super::message::ServerToSession>,
    }

    #[derive(Message)]
    #[rtype(result = "()")]
    pub struct LeaveAllRooms {
        pub room: String,
        pub uuid: Uuid,
        pub addr: Recipient<super::message::ServerToSession>,
    }

}