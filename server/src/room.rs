use actix::prelude::*;
use actix::WeakAddr;

#[allow(unused_imports)]
use log::{info, error, debug, warn, trace};

use std::collections::HashMap;

use crate::protocol::ChatMessage;
use crate::session::{ClientSession, SessionId};

pub type RoomId = String;

#[derive(Debug)]
pub struct Participant {
    pub session_id: SessionId,
    pub addr: WeakAddr<ClientSession>,
}

#[derive(Debug)]
pub struct DefaultRoom {
    id: RoomId,
    ephemeral: bool,
    history: Vec<ChatMessage>,
    participants: HashMap<SessionId, Participant>,
}

impl DefaultRoom {
    pub fn new(id: RoomId) -> Self {
        Self {
            id,
            ephemeral: false,
            history: Vec::with_capacity(10_000),
            participants: Default::default()
         }
    }
}

impl Actor for DefaultRoom {
    type Context = Context<Self>;
}

pub mod command {
    use actix::prelude::*;
    #[allow(unused_imports)]
    use log::{info, error, debug, warn, trace};

    use crate::protocol::ChatMessage;
    use crate::session::SessionId;
    use super::{message, DefaultRoom, Participant};

    // use crate::presence::{ AuthToken, PresenceService, ValidateRequest };

    #[derive(Message)]
    pub struct AddParticipant {
        pub participant: Participant,
    }

    impl Handler<AddParticipant> for DefaultRoom {
        type Result = ();

        fn handle(&mut self, command: AddParticipant, ctx: &mut Self::Context)  {
            let AddParticipant { participant } = command;
            trace!("Room {:?} adds {:?}", self.id, participant);
            // TODO: prevent duplicates
            participant
                .addr.upgrade().unwrap()
                .send(message::RoomToSession::Joined(self.id.clone(), ctx.address().downgrade()))
                .into_actor(self)
                .then(|_, _,_| fut::ok(()))
                .spawn(ctx);

            participant
                .addr.upgrade().unwrap()
                .send(message::RoomToSession::History{room: self.id.clone(), messages: self.history.clone() })
                .into_actor(self)
                .then(|_, _,_| fut::ok(()))
                .spawn(ctx);
            self.participants.insert(participant.session_id, participant);
            trace!("{:?} participants: {:?}", self.id, self.participants);
        }
    }

    #[derive(Message)]
    pub struct RemoveParticipant {
        pub session_id: SessionId,
    }

    impl Handler<RemoveParticipant> for DefaultRoom {
        type Result = ();

        fn handle(&mut self, command: RemoveParticipant, _ctx: &mut Self::Context)  {
            let RemoveParticipant {session_id} = command;
            if let Some(_participant) = self.participants.remove(&session_id) {
                debug!("successfully removed {} from {:?}", session_id, self.id);
                trace!("{:?} participants: {:?}", self.id, self.participants);
                trace!("{:?} is ephemeral {}", self.id, self.ephemeral);
            }
            else {
                warn!("{} was not a participant in {:?}", session_id, self.id);
            }
        }
    }

    #[derive(Debug, Message)]
    #[rtype(result = "Result<(), String>")]
    pub struct Forward {
        pub message: ChatMessage,
        pub sender: SessionId,
    }

    impl Handler<Forward> for DefaultRoom {
        type Result = MessageResult<Forward>;

        fn handle(&mut self, fwd: Forward, ctx: &mut Self::Context) -> Self::Result {
            info!("room {:?} received {:?}", self.id, fwd);

            let Forward { message, .. } = fwd;

            self.history.push(message.clone());

            for  participant in self.participants.values() {
                trace!("forwarding message to {:?}", participant);

                participant.addr.upgrade().unwrap()
                    .send(message::RoomToSession::ChatMessage {
                        message: message.clone(),
                        room: self.id.clone(),
                    })
                    .into_actor(self)
                    .then(|_, _slf, _| {
                        trace!("chatmessages passed on");
                        fut::ok(())
                    })
                    .spawn(ctx);
                }
            MessageResult(Ok(()))
        }
    }
}

pub mod message {
    use actix::prelude::*;
    use actix::WeakAddr;
    use crate::protocol::ChatMessage;

    use super::{DefaultRoom, RoomId};

    #[derive(Message, Debug)]
    #[rtype(result = "()")]
    pub enum RoomToSession {
        Joined(RoomId, WeakAddr<DefaultRoom>),
        ChatMessage {
            room: RoomId,
            message: ChatMessage,
        },
        History {
            room: RoomId,
            messages: Vec<ChatMessage>
        }
    }
}