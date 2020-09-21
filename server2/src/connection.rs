use futures::{
    select,
    sink::SinkExt,
    stream::{SplitSink, SplitStream},
    StreamExt,
};
use uuid::Uuid;
use warp::ws::{Message, WebSocket};
use xactor::{Actor, Context, StreamHandler};

use signaler_protocol::{ConnectionCommand, Credentials, SessionDescription, SessionMessage};

type WsSender = SplitSink<WebSocket, Message>;
type WsReceiver = SplitStream<WebSocket>;

pub struct Connection {
    connection_id: Uuid,
    ws_sender: WsSender,

    /// receiver on websocket
    /// this is taken out of here after starting by `Actor::started()`
    /// TODO: find a way to pass in the receiver without having to store it like this
    ws_receiver: Option<WsReceiver>,

    message_handler: Box<DynMessageHandler>,
}

pub(crate) type DynMessageHandler = dyn (Fn(&Connection, &str, &mut Context<Connection>)) + Send;

impl Connection {
    pub fn new(ws: WebSocket) -> Self {
        let (ws_sender, ws_receiver) = ws.split();
        Connection {
            connection_id: Uuid::new_v4(),
            ws_receiver: Some(ws_receiver),
            ws_sender,
            message_handler: Box::new(Self::handle_connection_message),
        }
    }

    async fn send(&mut self, msg: impl ToString) {
        let payload = msg.to_string();
        if let Err(e) = self.ws_sender.send(Message::text(&payload)).await {
            log::warn!("failed to send message on websocket {} {}", payload, e);
        }
    }

    fn handle_incoming_message(&self, raw_msg: &str, ctx: &mut Context<Self>) {
        let handler = &self.message_handler;
        handler(self, raw_msg, ctx);
    }

    fn handle_connection_message(&self, raw_msg: &str, ctx: &mut Context<Self>) {
        // parse as
        let parsed: Result<ConnectionCommand, _> = serde_json::from_str(raw_msg);
        if let Ok(msg) = parsed {
            log::trace!("parsed ok\n{}\n{:?}", raw_msg, msg);
            match msg {
                ConnectionCommand::Authenticate { credentials } => self.associate_session(credentials, ctx),
            }
        } else {
            log::warn!("cannot parse: {}", raw_msg);
            // log::debug!("suggestions:\n{}", SessionCommand::suggestions())
        }
    }

    // TODO: fn handle_session_message(&self, raw_msg: &str, ctx: &mut Context<Self>) {}

    fn associate_session(&self, _credentials: Credentials, _ctx: &mut Context<Self>) {}
}

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

#[async_trait::async_trait]
impl Actor for Connection {
    async fn started(&mut self, ctx: &mut xactor::Context<Self>) -> xactor::Result<()> {
        log::info!("starting connection on actor {:?}", ctx.actor_id());

        if let Some(ws_receiver) = self.ws_receiver.take() {
            ctx.add_stream(ws_receiver);
            self.send(
                SessionMessage::Welcome {
                    session: SessionDescription {
                        session_id: self.connection_id,
                    },
                }
                .into_json(),
            )
            .await;
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

pub mod commands {
    use super::*;
    use xactor::Handler;

    // #[xactor::message]
    // pub struct SayWelcome;

    // #[async_trait::async_trait]
    // impl Handler<SayWelcome> for Connection {
    //     async fn handle(&mut self, _ctx: &mut xactor::Context<Self>, _msg: SayWelcome) {
    //         self.send(r#"{"msg": "Welcome Friend" }"#).await
    //     }
    // }
}
