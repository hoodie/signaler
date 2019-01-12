#![allow(unused_imports)]
// extern crate actix;
use env_logger::{self, Env};
use log::{debug, info, trace, warn};

use actix_web::server::HttpServer;
use actix_web::{fs, ws, App, Error, HttpRequest, HttpResponse};

use std::{env, error};

pub mod protocol;
pub mod session;
pub mod server;

use crate::session::*;

const LOG_VAR: &str = "SIGNALIZER_LOG";
const BIND_TO: &str = "127.0.0.1:8080";

fn ws_route(req: &HttpRequest<()>) -> Result<HttpResponse, Error> {
    // debug!("chat route: {:?}", req);
    ws::start(req, ClientSession::default())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if env::var(LOG_VAR).is_err() {
        env::set_var(LOG_VAR, "signalizer=trace,actix_web=info");
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
