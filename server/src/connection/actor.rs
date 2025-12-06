use hannibal::{Actor, Context, Handler};
use tracing::log;

use super::Connection;
use crate::session::message::FromSession;

impl Actor for Connection {
    async fn started(&mut self, _ctx: &mut hannibal::Context<Self>) -> hannibal::DynResult<()> {
        log::trace!("starting on actor");

        // if let Some(ws_receiver) = self.ws_receiver.take() {
            // ctx.add_stream(ws_receiver);
            self.send_welcome().await;
        // } else {
        //     log::error!("unable to take ws_receiver stream");
        //     ctx.stop();
        // }
        Ok(())
    }
    async fn stopped(&mut self, _ctx: &mut hannibal::Context<Self>) {
        log::trace!("shutting down");
    }
}

impl Handler<FromSession> for Connection {
    async fn handle(&mut self, _ctx: &mut Context<Self>, msg: FromSession) {
        // log::debug!("received FromSession {:?}", &msg);
        match msg {
            FromSession::SessionMessage(session_msg) => {
                let payload = serde_json::to_string(&session_msg).unwrap();

                self.send(&payload).await;
            }
            FromSession::SessionAssociated { session } => {
                log::trace!("associated session");
                self.session = Some(session)
            }
        }
    }
}
