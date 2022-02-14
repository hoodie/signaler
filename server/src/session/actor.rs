use std::time::Duration;

use async_trait::async_trait;
use hannibal::{Actor, Handler};
use signaler_protocol::SessionMessage;
use tracing::log;

use crate::room::command::RoomToSession;

use super::{command::*, Session};

#[async_trait]
impl Actor for Session {
    async fn started(&mut self, ctx: &mut hannibal::Context<Self>) -> hannibal::Result<()> {
        log::info!("starting session on actor {:?}", ctx.actor_id());
        ctx.send_interval(Gc, Duration::from_secs(5));
        Ok(())
    }
    async fn stopped(&mut self, _ctx: &mut hannibal::Context<Self>) {
        log::debug!("shutting down Session");
    }
}

#[async_trait::async_trait]
impl Handler<Command> for Session {
    async fn handle(&mut self, ctx: &mut hannibal::Context<Self>, cmd: Command) {
        log::trace!("received command {:?}", cmd);
        self.dispatch_command(cmd.0, ctx).await;
    }
}

#[async_trait::async_trait]
impl Handler<RoomToSession> for Session {
    async fn handle(&mut self, _ctx: &mut hannibal::Context<Self>, msg: RoomToSession) {
        match msg {
            RoomToSession::Joined(room_id, room_addr) => {
                if self.rooms.insert(room_id.clone(), room_addr).is_some() {
                    log::warn!("received redundant Joined from {room_id:?}")
                }
            }
            RoomToSession::ChatMessage { room, message } => {
                self.send_to_connection(SessionMessage::Message { message, room }.into());
            }
        }
    }
}

#[async_trait::async_trait]
impl Handler<Gc> for Session {
    async fn handle(&mut self, ctx: &mut hannibal::Context<Self>, _: Gc) {
        self.gc(ctx);
    }
}
