use std::path::PathBuf;
use tracing::log;
use warp::{http::Uri, ws::WebSocket, Filter};
use xactor::{Actor, Context, Handler};

use std::net::SocketAddr;

pub async fn peer_connected(ws: WebSocket /*, broker: Broker*/) {
    log::debug!("user connected{:#?}", ws);
    let connection = crate::connection::Connection::new(ws);
    let addr = xactor::Actor::start(connection).await.unwrap();
    addr.wait_for_stop().await
}

#[derive(Default)]
pub struct WebServer;

#[async_trait::async_trait]
impl Actor for WebServer {
    async fn started(&mut self, _ctx: &mut xactor::Context<Self>) -> xactor::Result<()> {
        log::info!("started web server");
        Ok(())
    }
    async fn stopped(&mut self, _ctx: &mut xactor::Context<Self>) {
        log::info!("shutting down web server");
    }
}
// impl Service for WebServer {} // TODO: services aren't even supervised

#[async_trait::async_trait]
impl Handler<super::Listen> for WebServer {
    async fn handle(&mut self, _ctx: &mut Context<Self>, msg: super::Listen) {
        if let Err(error) = self.start(msg.socket).await {
            log::error!("{}", error);
        }
    }
}

impl WebServer {
    async fn start(&mut self, addr: SocketAddr) -> xactor::Result<()> {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let static_dir = || root.join("../static/");

        let routes = {
            let channel = warp::path("ws")
                .and(warp::ws())
                //.and(broker)
                .map(|ws: warp::ws::Ws /*, broker: Broker*/| {
                    #[allow(clippy::redundant_closure)]
                    ws.on_upgrade(move |socket| peer_connected(socket /*, broker*/))
                });

            let app = warp::path("app").and(warp::fs::dir(static_dir()));

            let redirect_to_app = warp::any().map(|| {
                log::trace!("redirecting");
                warp::redirect(Uri::from_static("/app/"))
            });

            let hello = warp::path("hello").map(|| {
                log::info!("✉️ hello world");
                "Hello, World!"
            });
            app.or(hello).or(channel).or(redirect_to_app)
        };

        log::info!("serving content from {}", static_dir().display());
        log::debug!("checking {} for availability", addr);

        let dummy_listener = std::net::TcpListener::bind(addr);
        match dummy_listener {
            Err(error) => log::error!("cannot bind {} because {}", addr, error),
            Ok(dummy_listener) => {
                std::mem::drop(dummy_listener);
                warp::serve(routes).run(addr).await;
            }
        }
        log::info!("web server has terminated");
        Ok(())
    }
}
