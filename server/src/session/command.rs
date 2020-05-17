use actix::prelude::*;
use actix::WeakAddr;
use actix_web_actors::ws::WebsocketContext;

#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

use super::ClientSession;
use crate::room::{command::UpdateParticipant, DefaultRoom};

#[derive(Message, Debug)]
// #[rtype(result = "Option<UserProfile>")]
#[rtype(result = "()")]
pub struct ProvideProfile<T: Actor> {
    pub room_addr: WeakAddr<T>,
}

impl Handler<ProvideProfile<DefaultRoom>> for ClientSession {
    type Result = MessageResult<ProvideProfile<DefaultRoom>>;

    fn handle(
        &mut self,
        p: ProvideProfile<DefaultRoom>,
        ctx: &mut WebsocketContext<Self>,
    ) -> Self::Result {
        if let Some(profile) = self.profile.clone() {
            if let Some(addr) = p.room_addr.upgrade() {
                addr.send(UpdateParticipant {
                    session_id: self.session_id,
                    profile,
                })
                .into_actor(self)
                .then(|_, _, _| fut::ready(()))
                .spawn(ctx);
            }
        } else {
            warn!(
                "{:?} was asked for profile, but didn't have one",
                self.session_id
            );
        }
        MessageResult(())
    }
}
