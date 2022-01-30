use async_trait::async_trait;
use tracing::log;
use xactor::{Actor, Handler};

use super::{command::*, Room};

#[async_trait]
impl Actor for Room {
    async fn started(&mut self, ctx: &mut xactor::Context<Self>) -> xactor::Result<()> {
        log::info!("starting Room {:?}", ctx.actor_id());
        Ok(())
    }
    async fn stopped(&mut self, _ctx: &mut xactor::Context<Self>) {
        log::trace!("shutting down Room");
    }
}

#[async_trait::async_trait]
impl Handler<Command> for Room {
    async fn handle(&mut self, _ctx: &mut xactor::Context<Self>, cmd: Command) {
        log::trace!("received command {:?}", cmd);
        match cmd {
            Command::AddParticipant { participant } => self.add_participant(participant),
        }
    }
}
