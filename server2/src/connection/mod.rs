use futures::{
    sink::SinkExt,
    stream::{SplitSink, SplitStream},
    StreamExt,
};
use tracing::log;
use uuid::Uuid;
use warp::ws::{Message, WebSocket};
use xactor::{Context, Service, WeakAddr};

use signaler_protocol::{ConnectionCommand, Credentials, SessionCommand, SessionDescription, SessionMessage};

use crate::{
    session::{self, Session},
    session_manager::{self, SessionManager},
};

mod actor;
pub mod command;
mod error;
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

    session: Option<WeakAddr<Session>>,
}

impl Connection {
    pub fn new(ws: WebSocket) -> Self {
        let connection_id = Uuid::new_v4();
        log::info!("new connection established {}", connection_id);
        let (ws_sender, ws_receiver) = ws.split();
        Connection {
            connection_id,
            ws_receiver: Some(ws_receiver),
            ws_sender,
            session: None,
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

    async fn handle_incoming_message(&mut self, raw_msg: &str, ctx: &mut Context<Self>) -> Result<(), error::Error> {
        if let Some(session) = self.session.as_ref() {
            let session = session.upgrade().ok_or(error::Error::SessionGone)?;
            let command = serde_json::from_str::<SessionCommand>(raw_msg)?;
            session.send(session::command::Command::from(command))?;
        } else {
            self.handle_connection_message(raw_msg, ctx).await?;
        }
        Ok(())
    }

    async fn handle_connection_message(&mut self, raw_msg: &str, ctx: &mut Context<Self>) -> Result<(), error::Error> {
        let msg = serde_json::from_str::<ConnectionCommand>(raw_msg)?;
        log::trace!("parsed ok {:?}", msg);
        match msg {
            ConnectionCommand::Authenticate { credentials } => self.associate_session(credentials, ctx).await,
        }
        Ok(())
    }

    async fn associate_session(&mut self, credentials: Credentials, ctx: &mut Context<Self>) {
        let sm = SessionManager::from_registry().await.unwrap();
        sm.send(session_manager::command::Command::AssociateConnection {
            credentials,
            connection: ctx.address().sender(),
        })
        .unwrap();
        self.send(signaler_protocol::SessionMessage::Authenticated.into_json())
            .await
    }
}
