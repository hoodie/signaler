mod warp;
use std::net::SocketAddr;

pub use self::warp::*;

#[xactor::message]
#[derive(Debug)]
pub struct Listen {
    pub socket: SocketAddr,
}
