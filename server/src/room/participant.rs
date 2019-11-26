use actix::prelude::*;
use actix::WeakAddr;

use crate::session::{ClientSession, SessionId};

#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

use std::convert::TryFrom;

#[derive(Debug)]
pub struct Participant {
    pub session_id: SessionId,
    pub addr: WeakAddr<ClientSession>,
}

#[derive(Debug)]
pub struct LiveParticipant {
    pub session_id: SessionId,
    pub addr: Addr<ClientSession>,
}

impl TryFrom<&Participant> for LiveParticipant {
    type Error = ();
    fn try_from(p: &Participant) -> Result<Self, Self::Error> {
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
