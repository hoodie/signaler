use tracing::log;

use prometheus::{Encoder, TextEncoder};
use warp::{http::Uri, ws::WebSocket, Filter};
use warp_prometheus::Metrics;

use hannibal::{Actor, Context, Handler, Service};

use std::{net::SocketAddr, path::PathBuf};

use crate::metrics::MetricsService;

pub async fn peer_connected(ws: WebSocket /*, broker: Broker*/) {
    log::debug!("user connected{:#?}", ws);
    let connection = crate::connection::Connection::new(ws);
    let addr = hannibal::Actor::start(connection).await.unwrap();
    addr.wait_for_stop().await
}

#[derive(Default)]
pub struct WebServer;

#[async_trait::async_trait]
impl Actor for WebServer {
    const NAME: &'static str = module_path!();

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
    #[tracing::instrument(level = tracing::Level::INFO, skip_all, name="warp_server")]
    async fn start(&mut self, addr: SocketAddr) -> hannibal::Result<()> {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let static_dir = || root.join("../static/");

        let registry = MetricsService::get_registry().await?;
        let path_labels = ["app", "ws", "metrics"];

        let _metrics = Metrics::new(&registry, &path_labels.into_iter().map(Into::into).collect());

        let routes = {
            let ws_route = warp::path("ws")
                .and(warp::ws())
                //.and(broker)
                .map(|ws: warp::ws::Ws /*, broker: Broker*/| {
                    #[allow(clippy::redundant_closure)]
                    ws.on_upgrade(move |socket| peer_connected(socket /*, broker*/))
                });

            let app_route = warp::path("app").and(warp::fs::dir(static_dir()));

            let metrics_route = warp::path("metrics").map(move || {
                let mut buffer = vec![];
                let encoder = TextEncoder::new();
                let metric_families = registry.gather();
                encoder.encode(&metric_families, &mut buffer).unwrap();
                let out: String = String::from_utf8_lossy(&buffer).into();
                out
            });

            let redirect_to_app = warp::any().map(|| {
                log::trace!("redirecting");
                warp::redirect(Uri::from_static("/app/"))
            });

            app_route.or(ws_route).or(metrics_route).or(redirect_to_app)
        };

        log::info!("serving content from {}", static_dir().display());
        log::debug!("checking {} for availability", addr);

        let dummy_listener = std::net::TcpListener::bind(addr);
        match dummy_listener {
            Err(error) => log::error!("cannot bind {} because {}", addr, error),
            Ok(dummy_listener) => {
                std::mem::drop(dummy_listener);
                warp::serve(
                    routes.with(warp::log::custom(|info| {
                        log::trace!(
                            "{} {} {} {:?}",
                            info.method(),
                            info.path(),
                            info.status(),
                            info.remote_addr()
                        )
                    })), //.with(warp::log::custom(move |log| metrics.http_metrics(log))),
                )
                .run(addr)
                .await;
            }
        }
        log::info!("web server has terminated");
        Ok(())
    }
}
