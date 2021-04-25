#![allow(unused_imports)]
use async_compat::{Compat, CompatExt};
use async_std::future::Future;
use env_logger::Env;

use async_std::prelude::*;
use std::env;

mod connection;
mod session;
mod warp_server;
mod tide_server;

const LOG_VAR: &str = "LOG";

#[async_std::main]
async fn main() {
    color_backtrace::install();

    if env::var(LOG_VAR).is_err() {
        env::set_var(LOG_VAR, "server2=trace,warp=info");
    }

    env_logger::init_from_env(Env::new().filter(LOG_VAR));

    let warp_server = async_std::task::spawn(async {
        let main_socket = std::net::SocketAddr::from(([0, 0, 0, 0], 3030));
        xactor::Supervisor::start(move || warp_server::WebServer::new(main_socket))
            // xactor::Supervisor::start(session::Session::default)
            .await
            .unwrap()
    });

    let tide_server = async_std::task::spawn(async {
                log::trace!("tide");
        use tide_websockets::{Message, WebSocket};
        let mut app = tide::new();

        app.at("/ws")
            .with(WebSocket::new(|_request, mut stream| async move {
                log::trace!("websocket request comming in");
                while let Some(Ok(Message::Text(input))) = stream.next().await {
                    let output: String = input.chars().rev().collect();

                    stream.send_string(format!("{} | {}", &input, &output)).await?;
                }
                log::trace!("websocket request closed");

                Ok(())
            }))
            .get(|_| async move { Ok("this was not a websocket request") });

        log::debug!("tide waiting");
        app.listen("0.0.0.0:8080").await.unwrap();
        log::debug!("tide closing");
    });

    let session = async_std::task::spawn(async { xactor::Supervisor::start(session::Session::default).await.unwrap() });

    let session2 =
        async_std::task::spawn(async { xactor::Supervisor::start(session::Session::default).await.unwrap() });

    futures::join!(tide_server, warp_server, session, session2);
}
