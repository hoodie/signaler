use actix::prelude::*;
use actix::utils::IntervalFunc;
use actix::WeakAddr;

#[allow(unused_imports)]
use log::{info, error, debug, warn, trace};

use std::collections::HashMap;
use std::time::Duration;

use crate::protocol;
use crate::session::{self, ClientSession, SessionId};

pub type RoomId = String;

#[derive(Debug)]
pub struct Participant {
    pub session_id: SessionId,
    pub addr: WeakAddr<ClientSession>,
}

#[derive(Debug)]
struct LiveParticipant {
    session_id: SessionId,
    addr: Addr<ClientSession>,
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

    fn live_participants<'a>(&'a self) -> impl Iterator<Item=LiveParticipant> + 'a {
        self.participants
            .values()
            .filter_map(|participant| {
                if let Some(addr) = participant.addr.upgrade() {
                    Some(LiveParticipant {
                        session_id: participant.session_id,
                        addr,
                    })
                } else {
                   error!("participant {} was dead, skipping", participant.session_id);
                   None
                }
            })
    }

    fn send_update_to_all_participants(&self, ctx: &mut Context<Self>) {

            for participant in self.live_participants() {
                trace!("forwarding message to {:?}", participant);

                participant
                    .addr
                    .send(message::RoomToSession::RoomState {
                        room: self.id.clone(),
                        participants: self.list_participants(ctx),
                    })
                    .into_actor(self)
                    .then(|_, _slf, _| {
                        trace!("RoomState passed on");
                        fut::ok(())
                    })
                    .spawn(ctx);
                }
    }

    fn gc(&mut self, _ctx: &mut Context<Self>) {
        self.clearout_dead_participants();
    }

    fn clearout_dead_participants(&mut self) {
        self.participants =
        self.participants
        .drain()
        .filter(|(_, participant)| {
            if participant.addr.upgrade().is_none() {
                debug!("garbage collecting participant {:?}", participant.session_id);
                false
            } else {
                true
            }
        })
        .collect()
    }


    fn lookup_presence_details(&self, _ctx: &mut Context<Self>) -> Vec<protocol::Participant> {
        // PresenceService.send... Lookup SessionIds
        self.live_participants()
            .filter_map(|participant| {
                participant
                    .addr
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

    fn started(&mut self, ctx: &mut Context<Self>) {
        IntervalFunc::new(Duration::from_millis(5_000), Self::gc)
                .finish()
                .spawn(ctx);
    }
}

pub mod command {
    use actix::prelude::*;
    #[allow(unused_imports)]
    use log::{info, error, debug, warn, trace};

    use crate::protocol;
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
            if let Some(addr) = participant.addr.upgrade() {
                addr
                    .send(message::RoomToSession::Joined(self.id.clone(), ctx.address().downgrade()))
                    .into_actor(self)
                    .then(|_, _,_| fut::ok(()))
                    .spawn(ctx);

                // TODO: do this on client demand
                addr
                    .send(message::RoomToSession::History{room: self.id.clone(), messages: self.history.clone() })
                    .into_actor(self)
                    .then(|_, _,_| fut::ok(()))
                    .spawn(ctx);

                self.participants.insert(participant.session_id, participant);

                self.send_update_to_all_participants(ctx);

                trace!("{:?} participants: {:?}", self.id, self.participants);
            } else {
                error!("participant address is cannot be upgraded {:?}", participant);
            }

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
            debug!("receive RemoveParticipant");
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
                                        trace!("room_manager says I'm fine to shut down");
                                        ctx.stop();
                                    },
                                    _ => {
                                        warn!("room_manager says it wasn't able to delete me 🤷")
                                    }
                                }

                                fut::ok(())
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
        fn handle(&mut self, _command: RoomUpdate, ctx: &mut Self::Context) -> Self::Result{
            MessageResult(self.list_participants(ctx))
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

        RoomState {
            room: RoomId,
            participants: Vec<Participant>
        },

        JoinDeclined {
            room: RoomId,
        }
    }

    }