use super::participant::RoomParticipant;

#[derive(Debug)]
#[xactor::message]
pub enum Command {
    AddParticipant { participant: RoomParticipant },
}
