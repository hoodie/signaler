//! Signaling Server
//!
//! Only on instance connecting many sessions
//!


use actix::prelude::*;
use log::*;
use uuid::Uuid;

use crate::protocol::ChatMessage;

use std::collections::{HashMap, HashSet};


pub type RoomId = String;
pub type SessionId = Uuid;

pub struct SignalingServer {
    sessions: HashMap<SessionId, Recipient<message::ServerToSession>>,
    rooms: HashMap<RoomId, HashSet<SessionId>>,
}

impl SignalingServer {
    pub fn print_state(&self) {
        let room_counts = self.rooms.iter().map(|(room, uuids)| (room, uuids.len())).collect::<HashMap<_,_>>();
        let sessions = self.sessions.keys().collect::<Vec<_>>();
        debug!("room members {:#?}", room_counts);
        debug!("sessions {:?}", sessions);
    }

    pub fn disconnect_session(&mut self, session_id: &SessionId) {

        if let Some(_) = self.sessions.remove(session_id)  {
            let mut rooms_to_notify = Vec::new();
            for (room, members) in &mut self.rooms {
                if members.remove(session_id) {
                    rooms_to_notify.push(room.to_owned())
                }
            }

            self.rooms = self.rooms.drain().filter(|(_, members)| members.len() > 0).collect();

            for room in &rooms_to_notify {
                debug!("somebody ({}) left {}", session_id, room);
                // TODO: send out notification
            }
        }

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
    use super::{RoomId, ChatMessage, SignalingServer, SessionId};

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
        pub session_id: SessionId,
        pub addr: Recipient<super::message::ServerToSession>,
    }

    impl Handler<JoinRoom> for SignalingServer {
        type Result = MessageResult<JoinRoom>;

        fn handle(&mut self, join: JoinRoom, _ctx: &mut Self::Context) -> Self::Result {
            self.sessions.insert(join.session_id, join.addr);
            if join.room.len() == 0 {
                error!("listname must'n be empty");
                return MessageResult(Err("listname must'n be empty".into()));
            }

            let newly_joined = self.rooms
                .entry(join.room.clone())
                .or_insert(Default::default())
                .insert(join.session_id);

            if newly_joined {
                debug!(
                    "rooms: {}, paricipants of {:?}",
                    serde_json::to_string_pretty(&self.rooms).unwrap(),
                    join.room
                    );
            } else {
                debug!("{} attempts to join {:?} again", join.session_id, join.room)
            }

            self.print_state();
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
        pub session_id: SessionId,
    }

    impl Handler<ListMyRooms> for SignalingServer {
        type Result = MessageResult<ListMyRooms>;

        fn handle(&mut self, me: ListMyRooms, _ctx: &mut Self::Context) -> Self::Result {
            info!("received listrequest from ...");
            MessageResult(
                self.rooms
                    .iter()
                    .filter(|(_room, participants)| participants.iter().any(|&session_id| session_id == me.session_id))
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
        pub session_id: SessionId,
        pub addr: Recipient<super::message::ServerToSession>,
    }

    #[derive(Message)]
    #[rtype(result = "()")]
    pub struct LeaveAllRooms {
        pub session_id: SessionId,
    }

    impl Handler<LeaveAllRooms> for SignalingServer {
        type Result = ();

        fn handle(&mut self, leave: LeaveAllRooms, _ctx: &mut Self::Context) -> Self::Result {
            debug!("SESSION LEAVING: {}", leave.session_id);
            self.print_state();
            self.disconnect_session(&leave.session_id);

            trace!("SESSION LEFT: {}", leave.session_id);
            self.print_state();
        }

    }

}