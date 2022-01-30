use signaler_protocol::RoomId;

use crate::room::participant::RoomParticipant;

#[xactor::message]
#[derive(Debug)]
pub enum Command {
    JoinRoom {
        room_id: RoomId,
        participant: RoomParticipant,
    },
}

#[xactor::message]
#[derive(Clone, Copy, Debug)]
pub struct Gc;
