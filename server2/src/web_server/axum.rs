use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        TypedHeader,
    },
    handler::get,
    http::StatusCode,
    response::IntoResponse,
    Router,
};
use tower_http::{
    services::ServeDir,
    trace::{DefaultMakeSpan, TraceLayer},
};
use xactor::{Actor, Context, Handler};

use std::net::SocketAddr;

pub async fn peer_connected(
    ws: WebSocketUpgrade,
    user_agent: Option<TypedHeader<headers::UserAgent>>,
) -> impl IntoResponse {
    tracing::trace!("peer connecting?");

    if let Some(TypedHeader(user_agent)) = user_agent {
        tracing::trace!("`{}` connected", user_agent.as_str());
    }

    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
    if let Some(msg) = socket.recv().await {
        if let Ok(msg) = msg {
            println!("Client says: {:?}", msg);
        } else {
            println!("client disconnected");
            return;
        }
    }

    loop {
        if socket.send(Message::Text(String::from("Hi!"))).await.is_err() {
            println!("client disconnected");
            return;
        }
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    }
}

#[derive(Default)]
pub struct WebServer;

#[async_trait::async_trait]
impl Actor for WebServer {
    async fn stopped(&mut self, _ctx: &mut xactor::Context<Self>) {
        log::info!("shutting down web server");
    }
}
// impl Service for WebServer {} // TODO: services aren't even supervised

#[async_trait::async_trait]
impl Handler<super::Listen> for WebServer {
    async fn handle(&mut self, _ctx: &mut Context<Self>, msg: super::Listen) {
        log::trace!("received {:?}", msg);
        if let Err(error) = self.start(msg.socket).await {
            log::error!("{}", error);
        }
    }
}

impl WebServer {
    async fn start(&mut self, socket: SocketAddr) -> xactor::Result<()> {
        async_compat::Compat::new(async {
            // Set the RUST_LOG, if it hasn't been explicitly defined

            // build our application with some routes
            let app = Router::new()
                .nest(
                    "/app",
                    axum::service::get(
                        ServeDir::new("../static")
                            //
                            .append_index_html_on_directories(true),
                    )
                    .handle_error(|error: std::io::Error| {
                        Ok::<_, std::convert::Infallible>((
                            StatusCode::INTERNAL_SERVER_ERROR,
                            format!("Unhandled internal error: {}", error),
                        ))
                    }),
                )
                // routes are matched from bottom to top, so we have to put `nest` at the
                // top since it matches all routes
                // .route("/ws", get(peer_connected))
                // logging so we can see whats going on
                .layer(TraceLayer::new_for_http().make_span_with(DefaultMakeSpan::default().include_headers(true)));

            // run it with hyper
            tracing::debug!("listening on {}", socket);
            axum::Server::bind(&socket)
                .serve(app.into_make_service())
                .await
                .unwrap();
        })
        .await;
        Ok(())
    }
}
