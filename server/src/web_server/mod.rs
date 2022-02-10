mod warp;
use std::net::SocketAddr;

pub use self::warp::*;

#[hannibal::message]
#[derive(Debug)]
pub struct Listen {
    pub socket: SocketAddr,
}
