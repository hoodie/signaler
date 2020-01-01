use actix::prelude::*;
use actix::utils::IntervalFunc;

#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

use std::collections::HashMap;
use std::time::Duration;

use crate::session::{self, ClientSession, SessionId};
use signaler_protocol as protocol;

pub type RoomId = String;

pub mod command;
pub mod message;
pub mod participant;

use self::participant::{LiveParticipant, Participant};

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
            participants: Default::default(),
        }
    }

    pub fn permanent(id: RoomId) -> Self {
        Self {
            id,
            ephemeral: false,
            history: Vec::with_capacity(10_000),
            participants: Default::default(),
        }
    }

    /*
    use std::convert::TryFrom;
    fn get_participant(&self, session_id: &SessionId) -> Option<LiveParticipant> {
        self.participants
            .get(session_id)
            .and_then(|p| LiveParticipant::try_from(p).ok())
    }
    */

    fn live_participants<'a>(&'a self) -> impl Iterator<Item = LiveParticipant> + 'a {
        self.participants.values().filter_map(|participant| {
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
                    participants: self.get_participant_profiles(ctx),
                })
                .into_actor(self)
                .then(|_, _slf, _| {
                    trace!("RoomState passed on");
                    fut::ready(())
                })
                .spawn(ctx);
        }
    }

    fn send_to_participant<'a, M>(
        &'a self,
        message: M,
        participant: &'a LiveParticipant,
        ctx: &'a mut Context<Self>,
    ) where
        M: Message + std::fmt::Debug + Send + 'static,
        <M as Message>::Result: Send,
        ClientSession: Handler<M>,
    {
        trace!("sending {:?} to {}", message, participant.session_id);
        participant
            .addr
            .send(message)
            .into_actor(self)
            .then(|_, _slf, _| fut::ready(()))
            .spawn(ctx);
    }

    fn gc(&mut self, _ctx: &mut Context<Self>) {
        self.participants = self
            .participants
            .drain()
            .inspect(|(_, participant)| {
                if participant.addr.upgrade().is_none() {
                    debug!(
                        "garbage collecting participant {:?}",
                        participant.session_id
                    );
                }
            })
            .filter(|(_, participant)| participant.addr.upgrade().is_some())
            .collect()
    }

    fn get_participant_profiles(&self, ctx: &mut Context<Self>) -> Vec<protocol::Participant> {
        // PresenceService.send... Lookup SessionIds
        self.live_participants().for_each(|participant| {
            let mut profiles: Vec<protocol::Participant> = Vec::new();
            participant
                .addr
                .send(session::command::ProvideProfile)
                .timeout(Duration::new(1, 0))
                .into_actor(self)
                .then(move |response, _slf, _ctx| {
                    match response {
                        Ok(maybe_profile) => {
                            if let Some(p) = maybe_profile {
                                profiles.push(protocol::Participant::from((
                                    p.into(),
                                    participant.session_id,
                                )));
                            } else {
                                warn!(
                                    "get_participant_profiles (room: {:?}) received empty profile",
                                    _slf.id
                                );
                            }
                        }
                        Err(err) => warn!("get_participant_profiles (room: {:?}) {}", _slf.id, err),
                    }
                    fut::ready(())
                })
                .spawn(ctx);
        });
        vec![]
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
