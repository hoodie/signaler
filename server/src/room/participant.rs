use actix::prelude::*;
use actix::WeakAddr;

use signaler_protocol as protocol;

use crate::session::{ClientSession, SessionId};
use crate::user_management::UserProfile;

#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

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
            error!("participant {} was dead, skipping", p.session_id);
            Err(())
        }
    }
}

impl Into<protocol::Participant> for &RosterParticipant {
    fn into(self) -> protocol::Participant {
        protocol::Participant {
            full_name: self.profile.as_ref().map(|p| p.full_name.to_string()).unwrap_or_else(|| String::from("unidentified")),
            session_id: self.session_id,
        }
    }
}
