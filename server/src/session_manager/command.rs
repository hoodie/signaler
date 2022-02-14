use hannibal::{message, WeakAddr};
use signaler_protocol::Credentials;

use crate::connection::Connection;

#[message]
pub enum Command {
    AssociateConnection {
        connection: WeakAddr<Connection>,
        credentials: Credentials,
    },
}

#[message]
#[derive(Clone, Copy, Debug)]
pub struct Gc;
