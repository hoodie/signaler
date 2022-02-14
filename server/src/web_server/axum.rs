use tracing::log;

// use prometheus::{Encoder, TextEncoder};

use axum::{routing::get_service, Router};
use tower_http::services::ServeDir;

use hannibal::{Actor, Context, Handler, Service};

use std::{net::SocketAddr, path::PathBuf};

use crate::metrics::MetricsService;

// pub async fn peer_connected(ws: WebSocket /*, broker: Broker*/) {
//     log::debug!("user connected{:#?}", ws);
//     let connection = crate::connection::Connection::new(ws);
//     let addr = hannibal::Actor::start(connection).await.unwrap();
//     addr.wait_for_stop().await
// }

async fn websocket_handler(ws: axum::extract::ws::WebSocketUpgrade) {
    ws.on_upgrade(|socket| async move {
        let connection = crate::connection::Connection::new(ws);
        let addr = hannibal::Actor::start(connection).await.unwrap();
        addr.wait_for_stop().await
    });
}

#[derive(Default)]
pub struct WebServer;

#[async_trait::async_trait]
impl Actor for WebServer {
    async fn started(&mut self, _ctx: &mut hannibal::Context<Self>) -> hannibal::Result<()> {
        log::info!("started web server");
        Ok(())
    }
    async fn stopped(&mut self, _ctx: &mut hannibal::Context<Self>) {
        log::info!("shutting down web server");
    }
}
impl Service for WebServer {} // TODO: services aren't even supervised

#[async_trait::async_trait]
impl Handler<super::Listen> for WebServer {
    async fn handle(&mut self, _ctx: &mut Context<Self>, msg: super::Listen) {
        if let Err(error) = self.start(msg.socket).await {
            log::error!("{}", error);
        }
    }
}

impl WebServer {
    async fn start(&mut self, addr: SocketAddr) -> hannibal::Result<()> {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let static_dir = root.join("../static/");

        let registry = MetricsService::get_registry().await?;
        let path_labels = ["app", "ws", "metrics"];

        // let _metrics = Metrics::new(&registry, &path_labels.into_iter().map(Into::into).collect());

        let app = Router::new()
            // ws route
            .route("/ws", axum::routing::get(websocket_handler))
            .route(
                "/",
                get_service(ServeDir::new(&static_dir))
                    // error handling
                    .handle_error(|error: std::io::Error| async move {
                        (
                            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                            format!("Unhandled internal server error: {}", error),
                        )
                    }),
            );

        log::info!("serving content from {}", static_dir.display());
        log::debug!("checking {} for availability", addr);

        let dummy_listener = std::net::TcpListener::bind(addr);
        match dummy_listener {
            Err(error) => log::error!("cannot bind {} because {}", addr, error),
            Ok(dummy_listener) => {
                std::mem::drop(dummy_listener);
                axum::Server::bind(&addr).serve(app.into_make_service()).await.unwrap()
            }
        }
        log::info!("web server has terminated");
        Ok(())
    }
}
