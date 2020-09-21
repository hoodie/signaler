use warp::ws::{Message, WebSocket};
use xactor::*;

use futures::{
    select,
    sink::SinkExt,
    stream::{SplitSink, SplitStream},
    StreamExt,
};

type WsSender = SplitSink<WebSocket, Message>;
type WsReceiver = SplitStream<WebSocket>;

pub struct Connection {
    pub ws_sender: WsSender,

    /// receiver on websocket
    pub ws_receiver: Option<WsReceiver>,
}

impl Connection {
    pub fn new(ws: WebSocket) -> Self {
        let (ws_sender, ws_receiver) = ws.split();
        Connection {
            ws_receiver: Some(ws_receiver),
            ws_sender,
        }
    }

    async fn send_to_remote(&mut self, msg: impl ToString) {
        let payload = msg.to_string();
        if let Err(e) = self.ws_sender.send(Message::text(&payload)).await {
            log::warn!("failed to send message on websocket {} {}", payload, e);
        }
    }
}

type WsStreamMessage = std::result::Result<warp::ws::Message, warp::Error>;

#[async_trait::async_trait]
impl StreamHandler<WsStreamMessage> for Connection {
    async fn handle(&mut self, ctx: &mut Context<Self>, received: WsStreamMessage) {
        match received {
            Ok(msg) => {
                log::trace!("received ws message {:#?}", msg);
                if msg.is_close() {
                    log::debug!("websocket disconnected");
                    ctx.stop(None);
                }

                if let Ok(content) = msg.to_str() {
                    log::info!("received {:?}", content);
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

#[async_trait::async_trait]
impl Actor for Connection {
    async fn started(&mut self, ctx: &mut xactor::Context<Self>) -> xactor::Result<()> {
        self.send_to_remote(r#"{"msg": "Welcome Friend" }"#).await;
        if let Some(ws_receiver) = self.ws_receiver.take() {
            ctx.add_stream(ws_receiver);
        } else {
            log::error!("unable to take ws_receiver stream");
            ctx.stop(None);
        }
        Ok(())
    }
    async fn stopped(&mut self, ctx: &mut xactor::Context<Self>) {
        log::info!("shutting down connection");
    }
}

pub mod commands {
    use super::*;

    #[xactor::message]
    pub struct SayWelcome;

    #[async_trait::async_trait]
    impl xactor::Handler<SayWelcome> for Connection {
        async fn handle(&mut self, _ctx: &mut xactor::Context<Self>, _msg: SayWelcome) {
            self.send_to_remote(r#"{"msg": "Welcome Friend" }"#).await
        }
    }
}
