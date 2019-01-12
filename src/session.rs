//! # Signaling Session
//!
//! One session per participant

// TODO: how to timeout sessions?

use ::actix::prelude::*;
use actix_web::ws::{self, WebsocketContext};

use log::{debug, info, trace, warn};

use crate::protocol::internal;
use crate::protocol::public::*;
use crate::server::SignalingServer;

#[derive(Default)]
pub struct ClientSession {
    // server_addr: Addr<self::server::SignalingServer>
}

impl ClientSession {

    /// parses raw string and passes it to `dispatch_incoming_message` or replies with error
    fn handle_incoming_message(&self, raw_msg: &str, ctx: &mut WebsocketContext<Self>) {
        // trace!("handle message: {:?}", raw_msg);
        let parsed: Result<SignalMessage, _> = serde_json::from_str(raw_msg);
        if let Ok(msg) = parsed {
            debug!("parsed ok\n{:#?}", msg);
            self.dispatch_incoming_message(msg, ctx)

        } else {
            let default_message = SignalMessage::err(format!("CantParse!\n{}", Self::allowed_messages()));
            warn!("cannot parse: {:#?}", raw_msg);
            trace!("allowed: {}", Self::allowed_messages());
            ctx.text(default_message.to_json());
        }
    }

    fn dispatch_incoming_message(&self, msg: SignalMessage, ctx: &mut WebsocketContext<Self>) {
        match msg.kind {
            SignalKind::ShutDown => {
                debug!("received shut down signal");
                System::current().stop();
            }
            SignalKind::ListRooms => {
                debug!("received list signal");
                self.talk_to_server(ctx)
            }
            _ => {}
        }
        ctx.text(SignalMessage::ok().to_json());
    }

    fn talk_to_server(&self, ctx: &mut WebsocketContext<Self>) {
        SignalingServer::from_registry()
            .send(internal::ListRooms)
            .into_actor(self)
            .then(|list, _, ctx| {
                debug!("list request answered: {:?}", list);
                match list {
                    Ok(list) => ctx.text(&SignalMessage::any(list).to_json()),
                    Err(error) => ctx.text(&SignalMessage::err(format!("{:#?}", error)).to_json())
                }
                fut::ok(())
            })
            .spawn(ctx);
    }

    fn allowed_messages() -> String {
        format!("{}\n{}",
                serde_json::to_string_pretty(&SignalMessage::join()).unwrap(),
                serde_json::to_string_pretty(&SignalMessage::list()).unwrap()
                )
    }

}

impl Actor for ClientSession {
    type Context = WebsocketContext<Self>;
    fn started(&mut self, _ctx: &mut Self::Context) {
        debug!("session started")
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        debug!("session stopped")
    }
}

impl StreamHandler<ws::Message, ws::ProtocolError> for ClientSession {
    fn handle(&mut self, msg: ws::Message, ctx: &mut Self::Context) {
        match msg {
            ws::Message::Ping(msg) => ctx.pong(&msg),
            ws::Message::Text(text) => {
                self.handle_incoming_message(&text, ctx);
            },
            _ => (),
        }
    }
}

