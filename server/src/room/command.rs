use actix::prelude::*;

use super::{message, participant::RosterParticipant, DefaultRoom};
use crate::{session::SessionId, user_management::UserProfile};
use signaler_protocol as protocol;

// use crate::presence::{ AuthToken, PresenceService, ValidateRequest };

#[derive(Message)]
#[rtype(result = "()")]
pub enum RoomCommand {
    AddParticipant {
        participant: RosterParticipant,
    },

    UpdateParticipant {
        profile: UserProfile,
        session_id: SessionId,
    },

    RemoveParticipant {
        session_id: SessionId,
    },

    Forward {
        message: protocol::ChatMessage,
        sender: SessionId,
    },

    GetParticipants {
        session_id: SessionId,
    },
}

impl Handler<RoomCommand> for DefaultRoom {
    type Result = ();

    fn handle(&mut self, command: RoomCommand, ctx: &mut Self::Context) {
        match command {
            RoomCommand::AddParticipant { participant } => {
                log::trace!("Room {:?} adds {:?}", self.id, participant);
                // TODO: prevent duplicates
                if let Some(addr) = participant.addr.upgrade() {
                    if let Err(error) = addr.try_send(message::RoomToSession::Joined(
                        self.id.clone(),
                        ctx.address().downgrade(),
                    )) {
                        log::error!(
                            "failed to confirm join to to client session {:?} {}",
                            participant,
                            error,
                        )
                    }

                    // TODO: do this on client demand
                    if let Err(error) = addr.try_send(message::RoomToSession::History {
                        room: self.id.clone(),
                        messages: self.history.clone(),
                    }) {
                        log::error!("failed to send history to participant {}", error)
                    }

                    self.roster.insert(participant.session_id, participant);

                    self.send_update_to_all_participants();

                    log::trace!("{:?} roster: {:?}", self.id, self.roster);
                } else {
                    log::error!("participant address is cannot be upgraded {:?}", participant);
                }
            }

            RoomCommand::UpdateParticipant { profile, session_id } => {
                log::trace!("Room {:?} updates {:?} with {:?}", self.id, session_id, profile);
                // if let Some(addr) = participant.addr.upgrade() {

                if let Some(roster_entry) = self.roster.get_mut(&session_id) {
                    roster_entry.profile = Some(profile);
                }

                self.send_update_to_all_participants();

                log::trace!("{:?} roster: {:?}", self.id, self.roster);
            }

            RoomCommand::RemoveParticipant { session_id } => self.remove_participant(&session_id, ctx),

            RoomCommand::Forward { message, .. } => {
                self.history.push(message.clone());

                for participant in self.live_participants() {
                    log::trace!("forwarding message to {:?}", participant);

                    if let Err(error) = participant.addr.try_send(message::RoomToSession::ChatMessage {
                        message: message.clone(),
                        room: self.id.clone(),
                    }) {
                        log::error!("failed to forward message to {:?} {}", participant, error)
                    }
                }
            }

            RoomCommand::GetParticipants { session_id } => self.send_roster_to_participant(&session_id),
        }
    }
}

#[derive(Debug, Message)]
#[rtype(result = "Result<ChatRoomCommandResult, String>")]
pub struct ChatRoomCommand {
    pub command: protocol::ChatRoomCommand,
    pub session_id: SessionId,
}

#[derive(Debug)]
pub enum ChatRoomCommandResult {
    Accepted,
    Rejected(String),
    NotImplemented(String),
}

impl Handler<ChatRoomCommand> for DefaultRoom {
    type Result = MessageResult<ChatRoomCommand>;

    fn handle(&mut self, fwd: ChatRoomCommand, ctx: &mut Self::Context) -> Self::Result {
        log::trace!("room {:?} received {:?}", self.id, fwd);

        let ChatRoomCommand { command, session_id } = fwd;
        log::trace!("received command from {:?}", session_id);

        match command {
            protocol::ChatRoomCommand::Message { content } => {
                match ctx.address().try_send(RoomCommand::Forward {
                    message: protocol::ChatMessage::new(content, session_id),
                    sender: session_id,
                }) {
                    Ok(_) => MessageResult(Ok(ChatRoomCommandResult::Accepted)),
                    Err(_) => MessageResult(Ok(ChatRoomCommandResult::Rejected("message".into()))),
                }
            }
            protocol::ChatRoomCommand::Leave => {
                self.remove_participant(&session_id, ctx);
                MessageResult(Ok(ChatRoomCommandResult::Accepted))
            }
            protocol::ChatRoomCommand::ListParticipants => {
                self.send_roster_to_participant(&session_id);
                MessageResult(Ok(ChatRoomCommandResult::Accepted))
            }
        }
    }
}
