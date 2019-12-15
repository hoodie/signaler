#![allow(unused_imports, dead_code)]

use futures_util::future;
use hyper::{
    header::{HeaderValue, UPGRADE},
    service::{make_service_fn, service_fn, Service},
    upgrade::Upgraded,
    Body, Request, Response, Server, StatusCode,
};
use tokio::{
    self,
    io::{AsyncRead, AsyncWrite},
    sync::{mpsc, Mutex},
};
use tracing::{debug, error, event, info, span, warn, Level};
use tracing_subscriber::FmtSubscriber;

use std::{
    collections::HashMap,
    convert::Infallible,
    sync::Arc,
    task::{Context, Poll},
};

struct FrontEnd {
    rooms: Vec<String>,
}

impl Default for FrontEnd {
    fn default() -> Self {
        Self {
            rooms: vec![String::from("default"), String::from("help")],
        }
    }
}

// impl Service<Request<Body>> for FrontEnd {
//     type Response = Response<Body>;
//     type Error = hyper::Error;
//     type Future = future::Ready<Result<Self::Response, Self::Error>>;
//
//     fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
//         Ok(()).into()
//     }

type EResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

async fn chat_frontend(req: Request<Body>) -> EResult<Response<Body>> {
    let path = req.uri().path();

    match path {
        "/ws" | "/ws/" => {
            info!("requested ws path {:?}", path);
            info!("{:#?}", req);
            spawn_upgrade(req).await
        }
        _ => {
            warn!("requested bad path {:?}", path);
            let response = Response::builder()
                .status(200)
                .body(Body::from("nothing to see"))
                .unwrap();
            Ok(response)
        }
    }
}
// }

async fn spawn_upgrade(req: Request<Body>) -> EResult<Response<Body>> {
    let mut res = Response::new(Body::empty());

    // Send a 400 to any request that doesn't have
    // an `Upgrade` header.
    if !req.headers().contains_key(UPGRADE) {
        *res.status_mut() = StatusCode::BAD_REQUEST;
        return Ok(res);
    }
    tokio::task::spawn(async move {
        match req.into_body().on_upgrade().await {
            Ok(upgraded) => {
                info!(r#"\0/ somebody wants to upgrade\n{:?}"#, upgraded);
                let _ = connect_websocket(upgraded).await;
            },
            Err(error) => {
                error!("failed to upgrade {}", error);
            }
        }
    });
    *res.status_mut() = StatusCode::SWITCHING_PROTOCOLS;
    res.headers_mut()
        .insert(UPGRADE, HeaderValue::from_static("foobar"));
    Ok(res)
}


async fn connect_websocket(upgraded: Upgraded) -> EResult<()> {
    use tokio::net::{TcpListener, TcpStream};
    use futures::{SinkExt, StreamExt};
    let mut socket = tokio_tungstenite::accept_async(upgraded).await?;
    while let Some(message) = socket.next().await {
        info!("received {:#?}", message);
    }
    Ok(())
}

type Tx = mpsc::UnboundedSender<String>;
type Rx = mpsc::UnboundedReceiver<String>;

#[derive(Default)]
struct Hub {
    peers: HashMap<String, Tx>,
}

impl Hub {
    pub fn new() -> Self {
        Default::default()
    }

    // async fn broadcast(&mut self, sender: SocketAddr, message: &str) {
    //     for peer in self.peers.iter_mut() {
    //         if *peer.0 != sender {
    //             let _ = peer.1.send(message.into());
    //         }
    //     }
    // }
}

struct Peer {}

impl Peer {}

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    color_backtrace::install();
    let my_subscriber = FmtSubscriber::new();
    tracing::subscriber::set_global_default(my_subscriber).expect("setting tracing default failed");

    let hub = Arc::new(Mutex::new(Hub::new()));

    // For every connection, we must make a `Service` to handle all
    // incoming HTTP requests on said connection.
    let mut service_count: u32 = 0;
    let make_svc = make_service_fn(|_conn| {
        service_count += 1;
        info!("Creating new Service (count: {})", service_count);
        async { Ok::<_, hyper::Error>(service_fn(chat_frontend)) }
    });

    let addr = ([127, 0, 0, 1], 3000).into();

    let server = Server::bind(&addr).serve(make_svc);

    println!("Listening on http://{}", addr);

    server.await?;

    Ok(())
}
