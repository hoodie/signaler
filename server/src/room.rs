use actix::prelude::*;
use actix::WeakAddr;

#[allow(unused_imports)]
use log::{info, error, debug, warn, trace};

use std::collections::HashMap;

use crate::protocol;
use crate::session::{self, ClientSession, SessionId};
use crate::user_management::UserProfile;

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
    history: Vec<protocol::ChatMessage>,
    // TODO: for privacy reasons the session_id should not be used as participant_id as well
    // because this id will be sent with every chat message
    participants: HashMap<SessionId, Participant>,
}


impl DefaultRoom {
    pub fn new(id: RoomId) -> Self {
        Self {
            id,
            ephemeral: true,
            history: Vec::with_capacity(10_000),
            participants: Default::default()
         }
    }

    pub fn permanent(id: RoomId) -> Self {
        Self {
            id,
            ephemeral: false,
            history: Vec::with_capacity(10_000),
            participants: Default::default()
         }
    }

    fn lookup_presence_details(&self, ctx: &mut Context<Self>) -> Vec<protocol::Participant> {
        use std::time::Duration;
        // PresenceService.send... Lookup SessionIds
        self.participants.values()
            .filter_map(|participant| {
                participant
                    .addr.upgrade().unwrap()
                    .send(session::command::ProvideProfile)
                    .timeout(Duration::new(1, 0))
                    .wait()
                    .map_err(|x| { error!("timeout requesting profile from ClientSession {}", participant.session_id); x })
                    .ok()
                    .map(|maybe_profile| maybe_profile.map(|p| protocol::Participant::from((p, participant.session_id))))
            })
            .filter_map(|x|x)
            .collect()
    }
    fn list_participants(&self, ctx: &mut Context<Self>) -> Vec<protocol::Participant> {
        self.lookup_presence_details(ctx)
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
    use crate::room_manager::RoomManagerService;
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

            // TODO: do this on client demand
            participant
                .addr.upgrade().unwrap()
                .send(message::RoomToSession::History{room: self.id.clone(), messages: self.history.clone() })
                .into_actor(self)
                .then(|_, _,_| fut::ok(()))
                .spawn(ctx);

            participant
                .addr.upgrade().unwrap()
                .send(message::RoomToSession::Participants {
                    room: self.id.clone(),
                    participants: self.list_participants(ctx),
                })
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

        fn handle(&mut self, command: RemoveParticipant, ctx: &mut Self::Context)  {
            let RemoveParticipant {session_id} = command;
            if let Some(_participant) = self.participants.remove(&session_id) {
                debug!("successfully removed {} from {:?}", session_id, self.id);
                trace!("{:?} participants: {:?}", self.id, self.participants);
                if self.participants.values().count() == 0 {
                    if self.ephemeral {
                        trace!("{:?} is empty and ephemeral => trying to stop {:?}", self.id, self);
                        RoomManagerService::from_registry()
                            .send(crate::room_manager::command::CloseRoom(self.id.clone()))
                            .into_actor(self)
                            .then(|success, _slf, ctx| {
                                match success {
                                    Ok(true) => {
                                        trace!("room_manager sais I'm fine to shut down");
                                        ctx.stop();
                                    },
                                    _ => {
                                        warn!("room_manager sais it wasn't able to delete me 🤷")
                                    }
                                }

                                fut::ok(())
                            })
                            .spawn(ctx)

                    } else {
                        trace!("{:?} is empty but not ephemeral, staying around", self.id);
                    }
                }
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
    use crate::protocol::{ChatMessage, Participant};

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
        },

        Participants {
            room: RoomId,
            participants: Vec<Participant>
        },

        JoinDeclined {
            room: RoomId,
        }
    }
}