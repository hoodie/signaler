use env_logger::{self, Env};
#[allow(unused_imports)]
use log::{info, error, debug, warn, trace};

use actix_web::HttpServer;
use actix_web::{
    middleware,
    guard, web,
    http::{header, StatusCode },
    App, Error, HttpRequest, HttpResponse,
};
use actix_files as fs;
use actix_web_actors::ws;

use std::env;

pub mod protocol;
pub mod session;
// pub mod server;
pub mod presence;
pub mod room;
pub mod room_manager;


pub mod user_management;
pub mod static_data;

use crate::session::*;

const LOG_VAR: &str = "SIGNALER_LOG";
const BIND_VAR: &str = "SIGNALER_BIND";
const BIND_TO: &str = "127.0.0.1:8080";

fn ws_route(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    debug!("chat route: {:?}", req);
    ws::start(ClientSession::default(), &req, stream)
}

fn not_found(_req: HttpRequest) -> Result<fs::NamedFile, Error> {
    warn!(target: "WEB_INTERFACE", "not found");
    Ok(fs::NamedFile::open("../static/404.html")?.set_status_code(StatusCode::NOT_FOUND))
}

fn favicon(_req: HttpRequest) -> Result<fs::NamedFile, Error> {
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

            .wrap(middleware::Logger::default())

            .service(web::resource("/ws/").route(web::get().to(ws_route)))
            .service(fs::Files::new("/app", "../static").show_files_listing())

            // .resource("/favicon.ico", |r| r.f(favicon))

            .service(web::resource("/favicon.ico").route(web::get().to(favicon)))
            .service(web::resource("/").route(web::get().to(|req: HttpRequest| {
                trace!("{:?}", req);
                HttpResponse::Found()
                    .header(header::LOCATION, "app/index.html")
                    .finish()
            })))

            .default_resource(|r| {
                r.route(web::get().to(not_found))
                    .route(
                        web::route()
                            .guard(guard::Not(guard::Get()))
                            .to(|| HttpResponse::MethodNotAllowed()),
                    )
            })

    });

    info!("listening on http://{}", bind_to);
    server().bind(bind_to)?.start();

    sys.run()?;
    info!("shutting down I guess");

    Ok(())
}
