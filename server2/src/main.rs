use dotenv::dotenv;
use env_logger::Env;

mod config;
mod connection;
mod session;
mod web_server;

use crate::config::Config;
use crate::web_server::WebServer;

#[async_std::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    color_backtrace::install();
    dotenv().unwrap();

    let config = dbg!(Config::from_env().unwrap());

    env_logger::init_from_env(Env::new().filter("LOG_CONFIG"));

    let session1 =
        async_std::task::spawn(async { xactor::Supervisor::start(session::Session::default).await.unwrap() });

    // let start_web_server = WebServer::from_registry().await?;
    let web_server = xactor::Supervisor::start(WebServer::default).await?;

    let _fo = futures::join!(
        session1,
        web_server.call(web_server::Listen {
            socket: ([0, 0, 0, 0], config.server.port).into(),
        })
    );

    Ok(())
}
