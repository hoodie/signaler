#![allow(unused_imports)]
use async_compat::{Compat, CompatExt};
use async_std::future::Future;
use env_logger::Env;

use async_std::prelude::*;
use std::env;

mod connection;
mod session;
mod web_server;

const LOG_VAR: &str = "LOG";

#[async_std::main]
async fn main() {
    color_backtrace::install();

    if env::var(LOG_VAR).is_err() {
        env::set_var(LOG_VAR, "server2=trace,warp=info");
    }

    env_logger::init_from_env(Env::new().filter(LOG_VAR));

    let server = async_std::task::spawn(async {
        let main_socket = std::net::SocketAddr::from(([0, 0, 0, 0], 3030));
        xactor::Supervisor::start(move || web_server::WebServer::new(main_socket))
            // xactor::Supervisor::start(session::Session::default)
            .await
            .unwrap()
    });
    let session = async_std::task::spawn(async {
        xactor::Supervisor::start(session::Session::default).await.unwrap()
    });

    let session2 = async_std::task::spawn(async {
        xactor::Supervisor::start(session::Session::default).await.unwrap()
    });

    futures::join!(server, session, session2);
}
