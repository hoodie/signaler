use env_logger::{self, Env};
#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

use actix_files as fs;
use actix_web::HttpServer;
use actix_web::{
    guard,
    http::{header, StatusCode},
    middleware, web, App, Error, HttpRequest, HttpResponse,
};
use actix_web_actors::ws;

use std::{env, path::PathBuf};

pub mod session;
// pub mod server;
pub mod presence;
pub mod room;
pub mod room_manager;

pub mod static_data;
pub mod user_management;

use crate::session::*;

const LOG_VAR: &str = "SIGNALER_LOG";
const BIND_VAR: &str = "SIGNALER_BIND";
const BIND_TO: &str = "127.0.0.1:8080";

async fn ws_route(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    debug!("chat route: {:?}", req);
    ws::start(ClientSession::default(), &req, stream)
}

async fn not_found(_req: HttpRequest) -> Result<fs::NamedFile, Error> {
    warn!(target: "WEB_INTERFACE", "not found");
    Ok(fs::NamedFile::open("../static/404.html")?.set_status_code(StatusCode::NOT_FOUND))
}

async fn favicon(_req: HttpRequest) -> Result<fs::NamedFile, Error> {
    Ok(fs::NamedFile::open("../static/favicon.ico")?)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    color_backtrace::install();
    if env::var(LOG_VAR).is_err() {
        env::set_var(LOG_VAR, "signaler=debug,actix_web=info");
    }
    env_logger::init_from_env(Env::new().filter(LOG_VAR));
    let bind_to = env::var(BIND_VAR).unwrap_or_else(|_| BIND_TO.into());

    let home = if option_env!("DOCKERIZE").is_some() {
        std::env::current_dir().unwrap()
    } else {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    };
    info!("running in {}", home.display());

    let sys = actix::System::new("signaler");

    let server = || {
        HttpServer::new(move || {
            App::new()
                // logger
                .wrap(middleware::Logger::default())
                // routes
                .service(web::resource("/ws/").route(web::get().to(ws_route)))
                .service(fs::Files::new("/app", home.join("../static")).index_file("index.html"))
                .service(
                    fs::Files::new("/app2", home.join("../webapp-svelte/public"))
                        .index_file("index.html"),
                )
                // statics
                .service(web::resource("/favicon.ico").route(web::get().to(favicon)))
                // redirects
                .service(web::resource("/").route(web::get().to(|req: HttpRequest| {
                    trace!("{:?}", req);
                    HttpResponse::Found()
                        .header(header::LOCATION, "app/index.html")
                        .finish()
                })))
                // fallback
                .default_service(
                    web::resource("").route(web::get().to(not_found)).route(
                        web::route()
                            .guard(guard::Not(guard::Get()))
                            .to(HttpResponse::MethodNotAllowed),
                    ),
                )
        })
    };

    info!("listening on http://{}", bind_to);
    server().bind(bind_to)?.run();

    sys.run()?;
    info!("shutting down I guess");

    Ok(())
}
