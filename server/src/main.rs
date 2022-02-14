use dotenv::dotenv;
use hannibal::Service;
use tracing::log;

mod config;
mod connection;
mod metrics;
mod room;
mod room_manager;
mod session;
mod session_manager;
mod web_server;

use crate::config::Config;
use crate::web_server::axum::WebServer;

#[async_std::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    color_backtrace::install();
    dotenv().unwrap();

    tracing_subscriber::fmt()
        // .pretty()
        // .with_thread_names(true)
        // enable everything
        .with_max_level(tracing::Level::TRACE)
        .with_env_filter(tracing_subscriber::EnvFilter::from_env("LOG_CONFIG"))
        // sets this to be the default, global collector for this application.
        .init();

    let config = Config::from_env().unwrap();
    log::info!("configured with {config:?}");

    log::debug!("log config {:?}", config.log_config);

    let server = match config.server.flavor {
        config::ServerFlavor::Warp => crate::web_server::warp::WebServer::from_registry().await?.caller(),
        config::ServerFlavor::Axum => crate::web_server::axum::WebServer::from_registry().await?.caller(),
    };

    server
        .call(web_server::Listen {
            socket: ([0, 0, 0, 0], config.server.port).into(),
        })
        .await?;

    Ok(())
}
