use xactor::Actor;

use super::Connection;

#[async_trait::async_trait]
impl Actor for Connection {
    async fn started(&mut self, ctx: &mut xactor::Context<Self>) -> xactor::Result<()> {
        log::info!("starting connection on actor {:?}", ctx.actor_id());

        if let Some(ws_receiver) = self.ws_receiver.take() {
            log::info!("sending welcome");
            ctx.add_stream(ws_receiver);
            self.send_welcome().await;
        } else {
            log::error!("unable to take ws_receiver stream");
            ctx.stop(None);
        }
        Ok(())
    }
    async fn stopped(&mut self, _ctx: &mut xactor::Context<Self>) {
        log::info!("shutting down connection");
    }
}
