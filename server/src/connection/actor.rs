use async_trait::async_trait;
use hannibal::{Actor, Context, Handler};
use signaler_protocol::SessionMessage;
use tracing::log;

use super::Connection;
use crate::session::message::FromSession;
use crate::session_manager::command::SessionAssociated;

#[async_trait::async_trait]
impl Actor for Connection {
    async fn started(&mut self, ctx: &mut hannibal::Context<Self>) -> hannibal::Result<()> {
        log::trace!("starting on actor {:?}", ctx.actor_id());

        if let Some(ws_receiver) = self.ws_receiver.take() {
            ctx.add_stream(ws_receiver);
            self.send_welcome().await;
        } else {
            log::error!("unable to take ws_receiver stream");
            ctx.stop(None);
        }
        Ok(())
    }
    async fn stopped(&mut self, _ctx: &mut hannibal::Context<Self>) {
        log::trace!("shutting down");
    }
}

#[async_trait]
impl Handler<SessionAssociated> for Connection {
    async fn handle(&mut self, _ctx: &mut Context<Self>, cmd: SessionAssociated) {
        log::trace!("associated session");
        self.session = Some(cmd.session)
    }
}

#[async_trait]
impl Handler<FromSession> for Connection {
    async fn handle(&mut self, _ctx: &mut Context<Self>, msg: FromSession) {
        log::debug!("received FromSession {:?}", &msg);
        let session_msg: SessionMessage = msg.into();
        let payload = serde_json::to_string(&session_msg).unwrap();
        
        self.send(&payload).await;
    }
}
