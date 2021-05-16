use actix::{WeakAddr, prelude::*};

use signaler_protocol as protocol;

use crate::{session::{ClientSession, SessionId}, user_management::UserProfile};

use std::convert::TryFrom;

#[derive(Debug)]
pub struct RosterParticipant {
    pub session_id: SessionId,
    pub addr: WeakAddr<ClientSession>,
    pub profile: Option<UserProfile>,
}

#[derive(Debug)]
pub struct LiveParticipant {
    pub session_id: SessionId,
    pub addr: Addr<ClientSession>,
}

impl TryFrom<&RosterParticipant> for LiveParticipant {
    type Error = ();
    fn try_from(p: &RosterParticipant) -> Result<Self, Self::Error> {
        if let Some(addr) = p.addr.upgrade() {
            Ok(LiveParticipant {
                session_id: p.session_id,
                addr,
            })
        } else {
            log::error!("participant {} was dead, skipping", p.session_id);
            Err(())
        }
    }
}

impl From<&RosterParticipant> for protocol::Participant {
    fn from(val: &RosterParticipant) -> Self {
        protocol::Participant {
            full_name: val
                .profile
                .as_ref()
                .map(|p| p.full_name.to_string())
                .unwrap_or_else(|| String::from("unidentified")),
            session_id: val.session_id,
        }
    }
}
