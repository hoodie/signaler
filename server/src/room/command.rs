use actix::prelude::*;
#[allow(unused_imports)]
use log::{info, error, debug, warn, trace};

use signaler_protocol as protocol;
use crate::session::SessionId;
use crate::room_manager::RoomManagerService;
use crate::user_management::UserProfile;
use super::participant::{RosterParticipant, LiveParticipant};
use super::{message, DefaultRoom};
use std::convert::TryFrom;

// use crate::presence::{ AuthToken, PresenceService, ValidateRequest };

#[derive(Message)]
#[rtype(result = "()")]
pub struct AddParticipant {
    pub participant: RosterParticipant,
}

impl Handler<AddParticipant> for DefaultRoom {
    type Result = ();

    fn handle(&mut self, command: AddParticipant, ctx: &mut Self::Context)  {
        let AddParticipant { participant } = command;
        trace!("Room {:?} adds {:?}", self.id, participant);
        // TODO: prevent duplicates
        if let Some(addr) = participant.addr.upgrade() {
            addr
                .send(message::RoomToSession::Joined(self.id.clone(), ctx.address().downgrade()))
                .into_actor(self)
                .then(|_, _,_| fut::ready(()))
                .spawn(ctx);

            // TODO: do this on client demand
            addr
                .send(message::RoomToSession::History{room: self.id.clone(), messages: self.history.clone() })
                .into_actor(self)
                .then(|_, _,_| fut::ready(()))
                .spawn(ctx);

            self.roster.insert(participant.session_id, participant);

            self.send_update_to_all_participants(ctx);

            trace!("{:?} roster: {:?}", self.id, self.roster);
        } else {
            error!("participant address is cannot be upgraded {:?}", participant);
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

    fn handle(&mut self, command: UpdateParticipant, ctx: &mut Self::Context)  {
        let UpdateParticipant { profile, session_id} = command;
        trace!("Room {:?} updates {:?} with {:?}", self.id, session_id, profile );
        // if let Some(addr) = participant.addr.upgrade() {

            if let Some(roster_entry) = self.roster.get_mut(&session_id) {
                roster_entry.profile = Some(profile);
            }

            self.send_update_to_all_participants(ctx);

            trace!("{:?} roster: {:?}", self.id, self.roster);
        // } else {
        //     error!("participant address is cannot be upgraded {:?}", participant);
        // }

    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct RemoveParticipant {
    pub session_id: SessionId,
}

impl Handler<RemoveParticipant> for DefaultRoom {
    type Result = ();

    fn handle(&mut self, command: RemoveParticipant, ctx: &mut Self::Context)  {
        let RemoveParticipant {session_id} = command;
        debug!("receive RemoveParticipant");
        if let Some(participant) = self.roster.remove(&session_id) {
            debug!("successfully removed {} from {:?}", session_id, self.id);
            trace!("{:?} roster: {:?}", self.id, self.roster);
            if let Ok(participant) = LiveParticipant::try_from(&participant) {
                self.send_to_participant(message::RoomToSession::Left { room: self.id.clone() }, &participant, ctx);
            }
            if self.roster.values().count() == 0 {
                if self.ephemeral {
                    trace!("{:?} is empty and ephemeral => trying to stop {:?}", self.id, self);
                    RoomManagerService::from_registry()
                        .send(crate::room_manager::command::CloseRoom(self.id.clone()))
                        .into_actor(self)
                        .then(|success, _slf, ctx| {
                            match success {
                                Ok(true) => {
                                    trace!("room_manager says I'm fine to shut down");
                                    ctx.stop();
                                },
                                _ => {
                                    warn!("room_manager says it wasn't able to delete me 🤷")
                                }
                            }

                            fut::ready(())
                        })
                        .spawn(ctx)

                } else {
                    trace!("{:?} is empty but not ephemeral, staying around", self.id);
                }
            } else {
                self.send_update_to_all_participants(ctx);
            }
        }
        else {
            warn!("{} was not a participant in {:?}", session_id, self.id);
        }
    }
}

#[derive(Message)]
#[rtype(result = "Vec<protocol::Participant>")]
pub struct RoomUpdate;

impl Handler<RoomUpdate> for DefaultRoom {
    type Result = MessageResult<RoomUpdate>;
    fn handle(&mut self, _command: RoomUpdate, _ctx: &mut Self::Context) -> Self::Result{
        MessageResult(self.get_roster())
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

    fn handle(&mut self, fwd: Forward, ctx: &mut Self::Context) -> Self::Result {
        info!("room {:?} received {:?}", self.id, fwd);

        let Forward { message, .. } = fwd;

        self.history.push(message.clone());

        for participant in self.live_participants() {
            trace!("forwarding message to {:?}", participant);

            participant
                .addr
                .send(message::RoomToSession::ChatMessage {
                    message: message.clone(),
                    room: self.id.clone(),
                })
                .into_actor(self)
                .then(|_, _slf, _| {
                    trace!("chatmessages passed on");
                    fut::ready(())
                })
                .spawn(ctx);
            }
        MessageResult(Ok(()))
    }
}