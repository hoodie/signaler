mod axum;
mod warp;
use std::net::SocketAddr;

// pub use self::warp::*;
pub use self::axum::*;

#[xactor::message]
#[derive(Debug)]
pub struct Listen {
    pub socket: SocketAddr,
}
