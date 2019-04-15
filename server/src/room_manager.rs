use actix::prelude::*;
use actix::WeakAddr;
#[allow(unused_imports)]
use log::{info, error, debug, warn, trace};

use std::collections::HashMap;

use crate::room::{DefaultRoom, Participant, RoomId};
use crate::room::command::AddParticipant;

#[derive(Copy, Clone, Debug)]
pub enum RoomManagerError {
    NoSuchRoom
}

#[derive(Default)]
pub struct RoomManagerService {
    pub rooms: HashMap<RoomId, Addr<DefaultRoom>>,
}

impl RoomManagerService {
    fn join_room(&mut self, name: &str, participant: Participant, ctx: &mut Context<Self>) {
        if let Some(room) = self.rooms.get(name) {
            trace!("found room {:?}, just join", name);
            // TODO: AWAOT!
            room.send(AddParticipant{participant}).into_actor(self)
                .then(|_res, _slf, _ctx| fut::ok(()))
                .spawn(ctx);
        } else {
            let room = self.create_room(name);
            trace!("no room found {:?}, create and then join {:#?}", name, self.list_rooms());
            room.upgrade().unwrap().send(AddParticipant{participant}).into_actor(self)
                .then(|_res, _slf, _ctx| fut::ok(()))
                .spawn(ctx);
        }

    }

    fn create_room(&mut self, name: &str) -> WeakAddr<DefaultRoom> {
        let room = DefaultRoom::new(name.into()).start();
        let weak_room = room.downgrade();
        self.rooms.insert(name.into(), room);
        weak_room
    }

    fn list_rooms(&self) -> Vec<String> {
        self.rooms.keys().cloned().collect()
    }
}

impl Actor for RoomManagerService {
    type Context = Context<Self>;
    fn started(&mut self, _ctx: &mut Self::Context) {
        debug!("RoomManager started");
    }
}

impl SystemService for RoomManagerService {}
impl Supervised for RoomManagerService {}

// messages

pub mod command {
    use actix::prelude::*;

    #[allow(unused_imports)]
    use log::{info, error, debug, warn, trace};

    use crate::room::Participant;
    use super::RoomManagerService;
    use crate::presence::{ AuthToken, PresenceService, ValidateRequest };

    #[derive(Message)]
    pub struct JoinRoom {
        pub room: String,
        pub participant: Participant,
        pub token: AuthToken,
    }


    impl Handler<JoinRoom> for RoomManagerService {
        type Result = ();

        fn handle(&mut self, request: JoinRoom, ctx: &mut Self::Context) -> Self::Result {
            trace!("RoomManagerService received request to join {:?}", request.room);
            let JoinRoom {room, participant, token} = request;
            // TODO: check token
            PresenceService::from_registry()
                .send(ValidateRequest { token })
                .into_actor(self)
                .then(move |is_valid, myself, ctx| {

                    match is_valid {
                        Ok(true)  => {
                            myself.join_room(&room, participant, ctx);
                        }
                        _ => {
                            warn!("{} attempted to join {} with invalid authentication", participant.session_id, room);
                            // TODO: send error to client_session
                        }
                    }

                    fut::ok(())
                })
                .spawn(ctx);
        }
    }

    #[derive(Message)]
    #[rtype(result = "Vec<String>")]
    pub struct ListRooms;


    impl Handler<ListRooms> for RoomManagerService {
        type Result = MessageResult<ListRooms>;

        fn handle(&mut self, _request: ListRooms, _ctx: &mut Self::Context) -> Self::Result {
            MessageResult(self.list_rooms())
        }
    }
}