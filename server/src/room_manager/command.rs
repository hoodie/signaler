use signaler_protocol::RoomId;

use crate::room::participant::RoomParticipant;

#[hannibal::message]
#[derive(Debug)]
pub enum Command {
    JoinRoom {
        room_id: RoomId,
        participant: RoomParticipant,
    },
}

#[hannibal::message]
#[derive(Clone, Copy, Debug)]
pub struct Gc;
