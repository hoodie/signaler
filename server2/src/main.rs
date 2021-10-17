use dotenv::dotenv;
use tracing::log;
use xactor::Service;

mod config;
mod connection;
mod metrics;
mod session;
mod session_manager;
mod web_server;

use crate::config::Config;
use crate::web_server::WebServer;

#[async_std::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    color_backtrace::install();
    dotenv().unwrap();

    let config = Config::from_env().unwrap();

    tracing_subscriber::fmt()
        // .pretty()
        .with_thread_names(true)
        // enable everything
        .with_max_level(tracing::Level::TRACE)
        .with_env_filter(tracing_subscriber::EnvFilter::from_env("LOG_CONFIG"))
        // sets this to be the default, global collector for this application.
        .init();

    log::debug!("log config {:?}", config.log_config);

    WebServer::from_registry()
        .await?
        .call(web_server::Listen {
            socket: ([0, 0, 0, 0], config.server.port).into(),
        })
        .await?;

    Ok(())
}
