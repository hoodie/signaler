use async_trait::async_trait;
use tracing::log;
use xactor::{Actor, Handler};

use super::{command::*, Session};

#[async_trait]
impl Actor for Session {
    async fn started(&mut self, ctx: &mut xactor::Context<Self>) -> xactor::Result<()> {
        log::info!("starting session on actor {:?}", ctx.actor_id());
        Ok(())
    }
}

#[async_trait::async_trait]
impl Handler<Foo> for Session {
    async fn handle(&mut self, _ctx: &mut xactor::Context<Self>, _msg: Foo) {
        log::info!("received a Foo");
    }
}

#[async_trait::async_trait]
impl Handler<Command> for Session {
    async fn handle(&mut self, _ctx: &mut xactor::Context<Self>, cmd: Command) {
        log::trace!("received command {:?}", cmd);
    }
}
