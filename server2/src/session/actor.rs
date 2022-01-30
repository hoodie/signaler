use std::time::Duration;

use async_trait::async_trait;
use tracing::log;
use xactor::{Actor, Handler};

use super::{command::*, Session};

#[async_trait]
impl Actor for Session {
    async fn started(&mut self, ctx: &mut xactor::Context<Self>) -> xactor::Result<()> {
        log::info!("starting session on actor {:?}", ctx.actor_id());
        ctx.send_interval(Gc, Duration::from_secs(5));
        Ok(())
    }
    async fn stopped(&mut self, _ctx: &mut xactor::Context<Self>) {
        log::trace!("shutting down Session");
    }
}

#[async_trait::async_trait]
impl Handler<Command> for Session {
    async fn handle(&mut self, _ctx: &mut xactor::Context<Self>, cmd: Command) {
        log::trace!("received command {:?}", cmd);
    }
}

#[async_trait::async_trait]
impl Handler<Gc> for Session {
    async fn handle(&mut self, ctx: &mut xactor::Context<Self>, _: Gc) {
        self.gc(ctx);
    }
}
