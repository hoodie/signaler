use async_compat::{Compat, CompatExt};
use futures::Future as _;
use warp::{http::Uri, ws::WebSocket, Filter};
use xactor::{Actor, Context, StreamHandler};

use std::net::SocketAddr;

pub async fn peer_connected(ws: WebSocket /*, broker: Broker*/) {
    log::debug!("user connected{:#?}", ws);
    let connection = crate::connection::Connection::new(ws);
    let addr = xactor::Actor::start(connection).await.unwrap();
    addr.wait_for_stop().await
}

pub struct WebServer {
    main_socket: SocketAddr,
}

impl WebServer {
    pub fn new(main_socket: SocketAddr) -> Self {
        WebServer { main_socket }
    }
}

#[async_trait::async_trait]
impl Actor for WebServer {
    async fn started(&mut self, ctx: &mut xactor::Context<Self>) -> xactor::Result<()> {
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
            app.or(hello)
                .or(channel)
                .or(redirect_to_app)
        };

        Compat::new(async {
            log::info!("starting web server on {:?}", self.main_socket);
            warp::serve(routes).run(self.main_socket).await;
            log::info!("web server has terminated");
        })
        .await;
        Ok(())
    }

    async fn stopped(&mut self, _ctx: &mut xactor::Context<Self>) {
        log::info!("shutting down web server");
    }
}
