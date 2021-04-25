use async_compat::{Compat, CompatExt};
use async_std::future::Future;
use env_logger::Env;

use async_std::prelude::*;
use xactor::{Actor, Context, StreamHandler};

use std::net::SocketAddr;

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
        async {
            log::trace!("tide");
            use tide_websockets::{Message, WebSocket};
            let mut app = tide::new();

            app.at("/ws")
                .with(WebSocket::new(|_request, mut stream| async move {
                    log::trace!("websocket request comming in");
                    while let Some(Ok(Message::Text(input))) = stream.next().await {
                        let output: String = input.chars().rev().collect();

                        stream.send_string(format!("{} | {}", &input, &output)).await?;
                    }
                    log::trace!("websocket request closed");

                    Ok(())
                }))
                .get(|_| async move { Ok("this was not a websocket request") });

            log::debug!("tide waiting");
            app.listen("0.0.0.0:8080").await.unwrap();
            log::debug!("tide closing");
        }
        .await;
        Ok(())
    }
}
