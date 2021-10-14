use async_trait::async_trait;
use xactor::{Actor, Context, Handler};

use super::Connection;
use crate::session_manager::command::SessionAssociated;

#[async_trait::async_trait]
impl Actor for Connection {
    async fn started(&mut self, ctx: &mut xactor::Context<Self>) -> xactor::Result<()> {
        log::trace!("starting connection on actor {:?}", ctx.actor_id());

        if let Some(ws_receiver) = self.ws_receiver.take() {
            ctx.add_stream(ws_receiver);
            self.send_welcome().await;
        } else {
            log::error!("unable to take ws_receiver stream");
            ctx.stop(None);
        }
        Ok(())
    }
    async fn stopped(&mut self, _ctx: &mut xactor::Context<Self>) {
        log::trace!("shutting down connection");
    }
}

#[async_trait]
impl Handler<SessionAssociated> for Connection {
    async fn handle(&mut self, _ctx: &mut Context<Self>, cmd: SessionAssociated) {
        log::trace!("associated session");
        self.session = Some(cmd.session)
    }
}
