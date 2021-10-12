use dotenv::dotenv;
use env_logger::Env;

mod config;
mod connection;
mod session;
mod web_server;

use crate::config::Config;
use crate::web_server::WebServer;

// #[async_std::main]
async fn xmain() -> Result<(), Box<dyn std::error::Error>> {
    color_backtrace::install();
    dotenv().unwrap();

    let config = dbg!(Config::from_env().unwrap());

    env_logger::init_from_env(Env::new().filter("LOG_CONFIG"));
    // tracing_subscriber::fmt::init();

    // let session1 =
    //     async_std::task::spawn(async { xactor::Supervisor::start(session::Session::default).await.unwrap() });

    // // let start_web_server = WebServer::from_registry().await?;
    // let web_server = xactor::Supervisor::start(WebServer::default).await?;

    // let _fo = futures::join!(
    //     session1,
    //     web_server.call(web_server::Listen {
    //         socket: ([0, 0, 0, 0], config.server.port).into(),
    //     })
    // );

    Ok(())
}

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        TypedHeader,
    },
    handler::get,
    http::StatusCode,
    response::{IntoResponse, Redirect},
    Router,
};
use std::{convert::Infallible, net::SocketAddr};
use tower_http::{
    compression::CompressionLayer,
    services::ServeDir,
    trace::{DefaultMakeSpan, TraceLayer},
};

async fn ws_handler(ws: WebSocketUpgrade, user_agent: Option<TypedHeader<headers::UserAgent>>) -> impl IntoResponse {
    if let Some(TypedHeader(user_agent)) = user_agent {
        println!("`{}` connected", user_agent.as_str());
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

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    // build our application with some routes
    let app = Router::new()
        // .route("/", get(|| async { Redirect::permanent("/app".parse().unwrap()) }))
        .nest(
            "/app",
            axum::service::get(ServeDir::new("../static")).handle_error(|error: std::io::Error| {
                Ok::<_, Infallible>((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Unhandled internal error: {}", error),
                ))
            }),
        )
        // routes are matched from bottom to top, so we have to put `nest` at the
        // top since it matches all routes
        .route("/ws", get(ws_handler))
        // logging so we can see whats going on
        .layer(TraceLayer::new_for_http().make_span_with(DefaultMakeSpan::default().include_headers(true)))
        .layer(CompressionLayer::new());

    // run it with hyper
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr).serve(app.into_make_service()).await.unwrap();
}
