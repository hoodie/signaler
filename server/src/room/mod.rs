//! Room Actor etc

use actix::prelude::*;

use std::{collections::HashMap, convert::TryFrom, time::Duration};

use crate::session::{self, ClientSession, SessionId};
use signaler_protocol as protocol;

pub type RoomId = String;

pub mod command;
pub mod message;
pub mod participant;

use self::participant::{LiveParticipant, RosterParticipant};

/// Holds a roster of Participants and distributes messages to `Participant`s
#[derive(Debug)]
pub struct DefaultRoom {
    id: RoomId,
    ephemeral: bool,
    history: Vec<protocol::ChatMessage>,
    // TODO: for privacy reasons the session_id should not be used as participant_id as well
    // because this id will be sent with every chat message
    roster: HashMap<SessionId, RosterParticipant>,
}

impl DefaultRoom {
    pub fn new(id: RoomId) -> Self {
        Self {
            id,
            ephemeral: true,
            history: Vec::with_capacity(10_000),
            roster: Default::default(),
        }
    }

    pub fn permanent(id: RoomId) -> Self {
        Self {
            id,
            ephemeral: false,
            history: Vec::with_capacity(10_000),
            roster: Default::default(),
        }
    }

    fn room_state(&self) -> message::RoomToSession {
        message::RoomToSession::RoomState {
            room: self.id.clone(),
            roster: self.get_roster(),
        }
    }

    fn get_roster(&self) -> Vec<protocol::Participant> {
        self.roster.values().map(Into::into).collect()
    }

    pub fn get_participant(&self, session_id: &SessionId) -> Option<LiveParticipant> {
        self.roster
            .get(session_id)
            .and_then(|p| LiveParticipant::try_from(p).ok())
    }

    fn live_participants<'a>(&'a self) -> impl Iterator<Item = LiveParticipant> + 'a {
        self.roster.values().filter_map(|participant| {
            if let Some(addr) = participant.addr.upgrade() {
                Some(LiveParticipant {
                    session_id: participant.session_id,
                    addr,
                })
            } else {
                log::error!("participant {} was dead, skipping", participant.session_id);
                None
            }
        })
    }

    fn send_update_to_all_participants(&self, ctx: &mut Context<Self>) {
        for participant in self.live_participants() {
            log::trace!("forwarding message to {:?}", participant);

            participant
                .addr
                .send(self.room_state())
                .into_actor(self)
                .then(|_, _slf, _| fut::ready(()))
                .spawn(ctx);
        }
    }

    fn send_to_participant<'a, M>(&'a self, message: M, participant: &'a LiveParticipant, ctx: &'a mut Context<Self>)
    where
        M: Message + std::fmt::Debug + Send + 'static,
        <M as Message>::Result: Send,
        ClientSession: Handler<M>,
    {
        log::trace!("sending {:?} to {}", message, participant.session_id);
        participant
            .addr
            .send(message)
            .into_actor(self)
            .then(|_, _slf, _| fut::ready(()))
            .spawn(ctx);
    }

    fn gc(&mut self, ctx: &mut Context<Self>) {
        let mut send_update = false;
        self.roster = self
            .roster
            .drain()
            .inspect(|(_, participant)| {
                if participant.addr.upgrade().is_none() {
                    log::debug!("garbage collecting participant {:?}", participant.session_id);
                    send_update = true;
                }
            })
            .filter(|(_, participant)| participant.addr.upgrade().is_some())
            .collect();

        if self.roster.is_empty() && self.ephemeral {
            log::debug!("empty ephemeral room {:?} - stopping", self.id);
            ctx.stop();
        }
        if send_update {
            self.send_update_to_all_participants(ctx);
        }
    }

    fn update_participant_profile(&mut self, participant: LiveParticipant, ctx: &mut Context<Self>) {
        let room_addr = ctx.address().downgrade();
        self.send_to_participant(session::command::ProvideProfile { room_addr }, &participant, ctx);
    }

    pub fn update_roster(&mut self, ctx: &mut Context<Self>) {
        #![allow(clippy::needless_collect)]
        let participants = self.live_participants().collect::<Vec<_>>();
        for participant in participants.into_iter() {
            self.update_participant_profile(participant, ctx);
        }
    }
}

impl Actor for DefaultRoom {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Context<Self>) {
        ctx.run_interval(Duration::from_millis(5_000), Self::gc);
        log::trace!("room {:?} started", self.id);
    }

    fn stopped(&mut self, _ctx: &mut Context<Self>) {
        log::trace!("room {:?} stopped", self.id);
    }
}
