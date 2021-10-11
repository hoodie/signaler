//! Signaler is a chat server that I use to explore actix for use in real time scenarios.

use env_logger::{self, Env};

use actix_files as fs;
use actix_web::HttpServer;
use actix_web::{
    guard,
    http::{header, StatusCode},
    middleware, web, App, Error, HttpRequest, HttpResponse,
};
use actix_web_actors::ws;
use dotenv::dotenv;

// use actix::WeakRecipient;

use std::{env, path::PathBuf};

mod config;
mod presence;
mod room;
mod session;
mod socket_connection;

mod static_data;

mod room_manager;
mod session_manager;
mod user_management;

//use crate::connection::ClientConnection;
use crate::config::Config;
use crate::session::*;
use crate::socket_connection::SocketConnection;

async fn ws_route(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    log::debug!("chat route: {:?}", req);
    ws::start(SocketConnection::default(), &req, stream)
}

async fn not_found(_req: HttpRequest) -> Result<fs::NamedFile, Error> {
    log::warn!(target: "WEB_INTERFACE", "not found");
    Ok(fs::NamedFile::open("../static/404.html")?.set_status_code(StatusCode::NOT_FOUND))
}

async fn favicon(_req: HttpRequest) -> Result<fs::NamedFile, Error> {
    Ok(fs::NamedFile::open("../static/favicon.ico")?)
}

#[actix::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    color_backtrace::install();
    dotenv().unwrap();

    let config = dbg!(Config::from_env().unwrap());

    env_logger::init_from_env(Env::new().filter("LOG_CONFIG"));

    let home = if option_env!("DOCKERIZE").is_some() {
        std::env::current_dir().unwrap()
    } else {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    };
    log::info!("running in {}", home.display());

    let server = HttpServer::new(move || {
        App::new()
            // logger
            .wrap(middleware::Logger::default())
            // routes
            .service(web::resource("/ws/").route(web::get().to(ws_route)))
            .service(fs::Files::new("/app", home.join("../static")).index_file("index.html"))
            .service(fs::Files::new("/app2", home.join("../webapp-svelte/public")).index_file("index.html"))
            // statics
            .service(web::resource("/favicon.ico").route(web::get().to(favicon)))
            // redirects
            .service(web::resource("/").route(web::get().to(|req: HttpRequest| {
                log::trace!("{:?}", req);
                HttpResponse::Found()
                    .append_header((header::LOCATION, "app/index.html"))
                    .finish()
            })))
            // fallback
            .default_service(web::route().guard(guard::Not(guard::Get())).to(not_found))
    });

    let bind_to = std::net::SocketAddr::new(config.server.host.parse().unwrap(), config.server.port);
    log::info!("listening on http://{}", bind_to);

    server.bind(bind_to).unwrap().run().await.unwrap();

    log::info!("shutting down I guess");

    Ok(())
}
