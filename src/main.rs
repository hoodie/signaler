use env_logger::{self, Env};
use log::*;

use actix_web::server::HttpServer;
use actix_web::{fs, ws, App, Error, HttpRequest, HttpResponse};

use std::env;

pub mod protocol;
pub mod session;
pub mod server;

use crate::session::*;

const LOG_VAR: &str = "SIGNALER_LOG";
const BIND_VAR: &str = "SIGNALER_BIND";
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
    let bind_to = env::var(BIND_VAR)
                .unwrap_or_else(|_| BIND_TO.into());

    let sys = actix::System::new("signalizer");

    let server = || HttpServer::new(move || {
        App::new()
            .resource("/ws/", |r| r.route().f(ws_route))
            .handler(
                "/",
                fs::StaticFiles::new("./static/")
                    .unwrap()
                    .index_file("index.html"),
            )
    });

    info!("listening on http://{}", bind_to);
    server().bind(bind_to)?.start();

    sys.run();
    info!("shutting down I guess");

    Ok(())
}
