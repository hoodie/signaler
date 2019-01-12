// extern crate actix;
use log::{debug, info, trace, warn};
use env_logger::{self, Env};

use ::actix::prelude::*;
use actix_web::server::HttpServer;
use actix_web::{fs, ws, App, Error, HttpRequest, HttpResponse};

use std::{env, error};

// mod server;
mod protocol;

use crate::protocol::*;

const LOG_VAR: &str = "SIGNALIZER_LOG";
const BIND_TO: &str = "127.0.0.1:8080";

#[derive(Default)]
struct SignalingSession {
    // server_addr: Addr<self::server::SignalingServer>
}

impl SignalingSession {
    fn handle_message(&self, raw_msg: &str, ctx: &mut actix_web::ws::WebsocketContext<SignalingSession>) {

        trace!("handle message: {:?}", raw_msg);
        let parsed: Result<SignalMessage, _> = serde_json::from_str(raw_msg);
        if let Ok(msg) = parsed {
            debug!("parsed: {:#?}", msg);
            match msg {
                SignalMessage{kind: SignalKind::ShutDown } => {
                    debug!("received shut down signal");
                    System::current().stop();
                }
                _ => {}
            }
            ctx.text(serde_json::to_string(&SignalMessage::ok()).unwrap());

        } else {
            let default_message = serde_json::to_string(&SignalMessage::err(format!("CantParse!\n{}", Self::allowed_messages()))).unwrap();
            warn!("cannot parse: {:#?}", raw_msg);
            ctx.text(default_message);
        }
    }

    fn allowed_messages() -> String {
        format!("{}\n{}",
                serde_json::to_string_pretty(&SignalMessage::join()).unwrap(),
                serde_json::to_string_pretty(&SignalMessage::list()).unwrap(),
                )
    }

}

impl Actor for SignalingSession {
    type Context = ws::WebsocketContext<Self>;
    fn started(&mut self, ctx: &mut Self::Context) {
        debug!("session started")
    }
}

impl StreamHandler<ws::Message, ws::ProtocolError> for SignalingSession {
    fn handle(&mut self, msg: ws::Message, ctx: &mut Self::Context) {
        match msg {
            ws::Message::Ping(msg) => ctx.pong(&msg),
            ws::Message::Text(text) => self.handle_message(&text, ctx),
            _ => (),
       }
    }

}

fn ws_route(req: &HttpRequest<()>) -> Result<HttpResponse, Error> {
    trace!("chat route: {:?}", req);
    ws::start(req, SignalingSession::default())
}

fn main() -> Result<(), Box<dyn std::error::Error>>{

    if env::var(LOG_VAR).is_err() {
        env::set_var(LOG_VAR, "signalizer=debug,actix_web=info");
    }

    env_logger::init_from_env(Env::new().filter(LOG_VAR));

    let sys = actix::System::new("signalizer");

    HttpServer::new(move || {
        App::new()
            .resource("/ws/", |r| r.route().f(ws_route))
            .handler(
                "/",
                fs::StaticFiles::new("./static/")
                    .unwrap()
                    .index_file("index.html"),
            )
    })
        .bind(BIND_TO)?
        .start();

    info!("listening on http://{}", BIND_TO);
    sys.run();
    info!("shutting down I guess");

    Ok(())
}
