use futures::{
    sink::SinkExt,
    stream::{SplitSink, SplitStream},
    StreamExt,
};
use uuid::Uuid;
use warp::ws::{Message, WebSocket};
use xactor::Context;

use signaler_protocol::{ConnectionCommand, Credentials, SessionDescription, SessionMessage};

mod actor;
mod command;
mod stream_handler;

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

    async fn send_welcome(&mut self) {
        self.send(
            SessionMessage::Welcome {
                session: SessionDescription {
                    session_id: self.connection_id,
                },
            }
            .into_json(),
        )
        .await;
    }

    fn handle_incoming_message(&self, raw_msg: &str, ctx: &mut Context<Self>) {
        let handler = &self.message_handler;
        handler(self, raw_msg, ctx);
    }

    fn handle_connection_message(&self, raw_msg: &str, ctx: &mut Context<Self>) {
        log::trace!("accepting connection message");
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
