use actix::prelude::*;
use protocol::ChatMessage;

use std::convert::TryFrom;

use super::{
    message,
    participant::{LiveParticipant, RosterParticipant},
    DefaultRoom,
};
use crate::{room_manager::RoomManagerService, session::SessionId, user_management::UserProfile};
use signaler_protocol as protocol;

// use crate::presence::{ AuthToken, PresenceService, ValidateRequest };

#[derive(Message)]
#[rtype(result = "()")]
pub struct AddParticipant {
    pub participant: RosterParticipant,
}

impl Handler<AddParticipant> for DefaultRoom {
    type Result = ();

    fn handle(&mut self, command: AddParticipant, ctx: &mut Self::Context) {
        let AddParticipant { participant } = command;
        log::trace!("Room {:?} adds {:?}", self.id, participant);
        // TODO: prevent duplicates
        if let Some(addr) = participant.addr.upgrade() {
            addr.try_send(message::RoomToSession::Joined(
                self.id.clone(),
                ctx.address().downgrade(),
            ))
            .unwrap();

            // TODO: do this on client demand
            addr.try_send(message::RoomToSession::History {
                room: self.id.clone(),
                messages: self.history.clone(),
            })
            .unwrap();

            self.roster.insert(participant.session_id, participant);

            self.send_update_to_all_participants();

            log::trace!("{:?} roster: {:?}", self.id, self.roster);
        } else {
            log::error!("participant address is cannot be upgraded {:?}", participant);
        }
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct UpdateParticipant {
    pub profile: UserProfile,
    pub session_id: SessionId,
}

impl Handler<UpdateParticipant> for DefaultRoom {
    type Result = ();

    fn handle(&mut self, command: UpdateParticipant, _ctx: &mut Self::Context) {
        let UpdateParticipant { profile, session_id } = command;
        log::trace!("Room {:?} updates {:?} with {:?}", self.id, session_id, profile);
        // if let Some(addr) = participant.addr.upgrade() {

        if let Some(roster_entry) = self.roster.get_mut(&session_id) {
            roster_entry.profile = Some(profile);
        }

        self.send_update_to_all_participants();

        log::trace!("{:?} roster: {:?}", self.id, self.roster);
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct RemoveParticipant {
    pub session_id: SessionId,
}

impl Handler<RemoveParticipant> for DefaultRoom {
    type Result = ();

    fn handle(&mut self, command: RemoveParticipant, ctx: &mut Self::Context) {
        let RemoveParticipant { session_id } = command;
        log::trace!("receive RemoveParticipant");
        if let Some(participant) = self.roster.remove(&session_id) {
            log::debug!("successfully removed {} from {:?}", session_id, self.id);
            log::trace!("{:?} roster: {:?}", self.id, self.roster);
            if let Ok(participant) = LiveParticipant::try_from(&participant) {
                self.send_to_participant(message::RoomToSession::Left { room: self.id.clone() }, &participant);
            }
            if self.roster.values().count() == 0 {
                if self.ephemeral {
                    log::trace!("{:?} is empty and ephemeral => trying to stop {:?}", self.id, self);
                    RoomManagerService::from_registry()
                        .send(crate::room_manager::command::CloseRoom(self.id.clone()))
                        .into_actor(self)
                        .then(|success, _slf, ctx| {
                            match success {
                                Ok(true) => {
                                    log::trace!("room_manager says I'm fine to shut down");
                                    ctx.stop();
                                }
                                _ => log::warn!("room_manager says it wasn't able to delete me 🤷"),
                            }

                            fut::ready(())
                        })
                        .spawn(ctx)
                } else {
                    log::trace!("{:?} is empty but not ephemeral, staying around", self.id);
                }
            } else {
                self.send_update_to_all_participants();
            }
        } else {
            log::warn!("{} was not a participant in {:?}", session_id, self.id);
        }
    }
}

#[derive(Message)]
#[rtype(result = "Vec<protocol::Participant>")]
pub struct RoomUpdate;

impl Handler<RoomUpdate> for DefaultRoom {
    type Result = MessageResult<RoomUpdate>;
    fn handle(&mut self, _command: RoomUpdate, _ctx: &mut Self::Context) -> Self::Result {
        MessageResult(self.get_roster())
    }
}

#[derive(Debug, Message)]
#[rtype(result = "Result<ChatRoomCommandResult, String>")]
pub struct ChatRoomCommand {
    pub command: protocol::ChatRoomCommand,
    pub sender: SessionId,
}

#[derive(Debug)]
pub enum ChatRoomCommandResult {
    Accepted,
    Rejected(String),
    NotImplemented(String),
}

impl Handler<ChatRoomCommand> for DefaultRoom {
    type Result = MessageResult<ChatRoomCommand>;

    fn handle(&mut self, fwd: ChatRoomCommand, _ctx: &mut Self::Context) -> Self::Result {
        log::trace!("room {:?} received {:?}", self.id, fwd);

        let ChatRoomCommand { command, sender } = fwd;
        log::trace!("received command from {:?}", sender);

        match command {
            // protocol::ChatRoomCommand::Join { room } => {}
            // protocol::ChatRoomCommand::Leave { room } => {}
            protocol::ChatRoomCommand::Message { content } => {
                match _ctx.address().try_send(Forward {
                    message: ChatMessage::new(content, sender),
                    sender,
                }) {
                    Ok(_) => MessageResult(Ok(ChatRoomCommandResult::Accepted)),
                    Err(_) => MessageResult(Ok(ChatRoomCommandResult::Rejected("message".into()))),
                }
            }
            // protocol::ChatRoomCommand::ListParticipants { room } => {}
            _ => MessageResult(Ok(ChatRoomCommandResult::NotImplemented("".into()))),
        }
    }
}

#[derive(Debug, Message)]
#[rtype(result = "Result<(), String>")]
pub struct Forward {
    pub message: protocol::ChatMessage,
    pub sender: SessionId,
}

impl Handler<Forward> for DefaultRoom {
    type Result = MessageResult<Forward>;

    fn handle(&mut self, fwd: Forward, _ctx: &mut Self::Context) -> Self::Result {
        log::trace!("room {:?} received {:?}", self.id, fwd);

        let Forward { message, .. } = fwd;

        self.history.push(message.clone());

        for participant in self.live_participants() {
            log::trace!("forwarding message to {:?}", participant);

            participant
                .addr
                .try_send(message::RoomToSession::ChatMessage {
                    message: message.clone(),
                    room: self.id.clone(),
                })
                .unwrap();
        }
        MessageResult(Ok(()))
    }
}
