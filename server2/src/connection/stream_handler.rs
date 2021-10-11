use xactor::{Context, StreamHandler};

use super::Connection;

type WsStreamMessage = std::result::Result<warp::ws::Message, warp::Error>;

#[async_trait::async_trait]
impl StreamHandler<WsStreamMessage> for Connection {
    async fn handle(&mut self, ctx: &mut Context<Self>, received: WsStreamMessage) {
        match received {
            Ok(msg) => {
                if msg.is_close() {
                    log::debug!("websocket disconnected");
                    ctx.stop(None);
                } else if let Ok(content) = msg.to_str() {
                    log::info!("received {:?}", content);
                    self.handle_incoming_message(content, ctx);
                } else {
                    log::error!("received invalid message {:?}", msg);
                    ctx.stop(Some(anyhow::anyhow!("unparsable message")));
                }
            }
            Err(err) => {
                log::warn!("received ws error {}", err);
                ctx.stop(Some(err.into()));
            }
        }
    }
}
