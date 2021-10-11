//! Room Actor etc

use actix::prelude::*;

use std::{collections::HashMap, convert::TryFrom, time::Duration};

use crate::{
    room_manager::RoomManagerService,
    session::{self, ClientSession, SessionId},
};
use signaler_protocol as protocol;
pub use signaler_protocol::RoomId;

mod command;

pub mod message;
pub mod participant;

pub use command::ChatRoomCommand;
pub use command::RoomCommand as Command;

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

    #[allow(clippy::needless_lifetimes)]
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

    fn send_update_to_all_participants(&self) {
        for participant in self.live_participants() {
            self.send_to_participant(self.room_state(), &participant);
        }
    }

    fn send_to_participant<'a, M>(&'a self, message: M, participant: &'a LiveParticipant)
    where
        M: Message + std::fmt::Debug + Send + 'static,
        <M as Message>::Result: Send,
        ClientSession: Handler<M>,
    {
        log::trace!("sending {:?} to {}", message, participant.session_id);
        if let Err(error) = participant.addr.try_send(message) {
            log::error!("failed to send to participant {:?} {}", participant, error);
        };
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
            self.send_update_to_all_participants();
        }
    }

    fn remove_participant(&mut self, session_id: &SessionId, ctx: &mut Context<Self>) {
        log::trace!("receive RemoveParticipant");
        if let Some(participant) = self.roster.remove(session_id) {
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

    fn send_roster_to_participant(&mut self, session_id: &SessionId) {
        if let Some(participant) = self.get_participant(session_id) {
            let room = self.id.clone();
            let roster = self.get_roster();

            self.send_to_participant(message::RoomToSession::RoomState { room, roster }, &participant)
        }
    }

    fn update_participant_profile(&mut self, participant: LiveParticipant, ctx: &mut Context<Self>) {
        let room_addr = ctx.address().downgrade();
        self.send_to_participant(session::command::ProvideProfile { room_addr }, &participant);
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
