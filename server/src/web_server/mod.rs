use std::net::SocketAddr;

pub mod axum;
pub mod warp;

#[hannibal::message]
#[derive(Debug)]
pub struct Listen {
    pub socket: SocketAddr,
}
