use actix::prelude::*;
use actix::WeakAddr;
#[allow(unused_imports)]
use log::{info, error, debug, warn, trace};

use std::collections::HashMap;

use crate::room::{
    DefaultRoom, RoomId,
    participant::Participant,
    message::RoomToSession,
    command::AddParticipant,
};

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
            room.send(AddParticipant{participant})
                .into_actor(self)
                .then(|_res, _slf, _ctx| fut::ready(()))
                .spawn(ctx);
        } else {
            let room = self.create_room(name);
            trace!("no room found {:?}, create and then join {:#?}", name, self.list_rooms());
            room.upgrade().unwrap().send(AddParticipant{participant}).into_actor(self)
                .then(|_res, _slf, _ctx| fut::ready(()))
                .spawn(ctx);
        }

    }

    fn send_decline(&mut self, room_id: &str, participant: Participant, ctx: &mut Context<Self>) {
        participant
            .addr.upgrade().unwrap()
            .send(RoomToSession::JoinDeclined { room: room_id.into()})
            .into_actor(self)
            .then(|_, _, _| fut::ready(()))
            .spawn(ctx);
    }

    fn create_room(&mut self, name: &str) -> WeakAddr<DefaultRoom> {
        trace!("create room: {:?}", name);
        let room = DefaultRoom::new(name.into()).start();
        let weak_room = room.downgrade();
        self.rooms.insert(name.into(), room);
        weak_room
    }

    fn create_permanent_room(&mut self, name: &str) -> WeakAddr<DefaultRoom> {
        trace!("create permanent room: {:?}", name);
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
        debug!("RoomManager started");
        self.create_permanent_room("default");
    }
}

impl SystemService for RoomManagerService {}
impl Supervised for RoomManagerService {}

// messages

pub mod command {
    use actix::prelude::*;

    #[allow(unused_imports)]
    use log::{info, error, debug, warn, trace};

    use crate::presence::{AuthToken, PresenceService, ValidateRequest };
    use crate::room::{participant::Participant, RoomId};
    use super::RoomManagerService;

    #[derive(Message)]
    #[rtype(result = "()")]
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
                .then(move |is_valid, slf, ctx| {

                    match is_valid {
                        Ok(true)  => {
                            slf.join_room(&room, participant, ctx);
                        }
                        _ => {
                            warn!("{} attempted to join {} with invalid authentication", participant.session_id, room);
                            slf.send_decline(&room, participant, ctx);
                            // TODO: send error to client_session
                        }
                    }

                    fut::ready(())
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

    #[derive(Message)]
    #[rtype(result = "bool")]
    pub struct CloseRoom (pub RoomId);


    impl Handler<CloseRoom> for RoomManagerService {
        type Result = MessageResult<CloseRoom>;

        fn handle(&mut self, CloseRoom(room_id): CloseRoom, _ctx: &mut Self::Context) -> Self::Result {
            // let CloseRoom(room_id) = request;
            trace!("received CloseRoom({:?})", room_id);
            MessageResult(self.close_room(room_id))
        }
    }
}