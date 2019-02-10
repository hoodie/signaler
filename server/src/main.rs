use env_logger::{self, Env};
use log::*;

use actix_web::server::HttpServer;
use actix_web::{
    fs, ws,
    middleware, pred, 
    http::{header, StatusCode, Method },
    App, Error, HttpRequest, HttpResponse,
};

use std::env;

pub mod protocol;

pub mod session;
pub mod server;
pub mod presence;

pub mod user_management;

use crate::session::*;

const LOG_VAR: &str = "SIGNALER_LOG";
const BIND_VAR: &str = "SIGNALER_BIND";
const BIND_TO: &str = "127.0.0.1:8080";

fn ws_route(req: &HttpRequest<()>) -> Result<HttpResponse, Error> {
    debug!("chat route: {:?}", req);
    ws::start(req, ClientSession::default())
}

fn not_found(_req: &HttpRequest) -> Result<fs::NamedFile, Error> {
    warn!(target: "WEB_INTERFACE", "not found");
    Ok(fs::NamedFile::open("../static/404.html")?.set_status_code(StatusCode::NOT_FOUND))
}

fn favicon(_req: &HttpRequest) -> Result<fs::NamedFile, Error> {
    Ok(fs::NamedFile::open("../static/favicon.ico")?)
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
            .resource("/bad", |r| {
                r.method(Method::GET).f(not_found);
                r.route().filter(pred::Not(pred::Get())).f(|_req| {
                    warn!("bad");
                    HttpResponse::MethodNotAllowed()
                });
            })

            .default_resource(|r| {
                r.method(Method::GET).f(not_found);
                r.route().filter(pred::Not(pred::Get())).f(|_req| HttpResponse::MethodNotAllowed());
            })

            .middleware(middleware::Logger::default())

            .resource("/ws/", |r| r.route().f(ws_route))

            .resource("/favicon.ico", |r| r.f(favicon))
            .handler(
                "/app",
                fs::StaticFiles::new("../static/")
                .unwrap()
                .index_file("index.html"),
                )

            .resource("/", |r| r.method(Method::GET).f(|req| {
                println!("{:?}", req);
                HttpResponse::Found()
                    .header(header::LOCATION, "app/index.html")
                    .finish()
            }))
    });

    info!("listening on http://{}", bind_to);
    server().bind(bind_to)?.start();

    sys.run();
    info!("shutting down I guess");

    Ok(())
}
