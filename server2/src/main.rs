#![allow(unused_imports)]
use async_compat::{Compat, CompatExt};
use env_logger::Env;
use warp::{http::Uri, ws::WebSocket, Filter};

use std::env;

mod connection;

const LOG_VAR: &str = "LOG";
async fn peer_connected(ws: WebSocket /*, broker: Broker*/) {
    log::debug!("user connected{:#?}", ws);
    let connection = connection::Connection::new(ws);
    let addr = xactor::Actor::start(connection).await.unwrap();
    addr.wait_for_stop().await
}

fn main() {
    color_backtrace::install();

    if env::var(LOG_VAR).is_err() {
        env::set_var(LOG_VAR, "server2=trace,warp=info");
    }

    env_logger::init_from_env(Env::new().filter(LOG_VAR));

    let routes = {
        let channel = warp::path("ws")
            .and(warp::ws())
            //.and(broker)
            .map(|ws: warp::ws::Ws /*, broker: Broker*/| {
                #[allow(clippy::redundant_closure)]
                ws.on_upgrade(move |socket| peer_connected(socket /*, broker*/))
            });
        let app = warp::path("app").and(warp::fs::dir("../static/"));
        let redirect_to_app = warp::any().map(|| warp::redirect(Uri::from_static("/app/")));

        let hello = warp::path("hello").map(|| {
            log::info!("✉️ hello world");
            "Hello, World!"
        });
        app.or(hello).or(channel).or(redirect_to_app)
    };

    let main_socket = std::net::SocketAddr::from(([0, 0, 0, 0], 3030));

    smol::block_on(Compat::new(async {
        warp::serve(routes).run(main_socket).await;
    }));
}
