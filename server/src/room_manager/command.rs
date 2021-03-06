use actix::prelude::*;

#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

use super::RoomManagerService;

use crate::{
    presence::{
        command::{AuthToken, ValidateRequest},
        PresenceService,
    },
    room::{participant::RosterParticipant, RoomId},
};

#[derive(Message)]
#[rtype(result = "()")]
pub enum RoomManagerCommand {
    JoinRoom {
        room: String,
        participant: RosterParticipant,
        token: AuthToken,
    },
}

impl Handler<RoomManagerCommand> for RoomManagerService {
    type Result = ();

    fn handle(&mut self, request: RoomManagerCommand, ctx: &mut Self::Context) -> Self::Result {
        let RoomManagerCommand::JoinRoom {
            room,
            participant,
            token,
        } = request;

        log::trace!("RoomManagerService received request to join {:?}", room);

        // TODO: check token
        PresenceService::from_registry()
            .send(ValidateRequest { token })
            .into_actor(self)
            .then(move |is_valid, slf, _ctx| {
                match is_valid {
                    Ok(true) => {
                        slf.join_room(&room, participant);
                    }
                    _ => {
                        log::warn!(
                            "{} attempted to join {} with invalid authentication",
                            participant.session_id,
                            room
                        );
                        slf.send_decline(&room, participant);
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

#[derive(Message, Debug)]
#[rtype(result = "bool")]
pub struct CloseRoom(pub RoomId);

impl Handler<CloseRoom> for RoomManagerService {
    type Result = MessageResult<CloseRoom>;

    fn handle(&mut self, room_id: CloseRoom, _ctx: &mut Self::Context) -> Self::Result {
        // let CloseRoom(room_id) = request;
        log::trace!("received {:?}", room_id);
        MessageResult(self.close_room(room_id.0))
    }
}
