use signaler_protocol::Credentials;
use hannibal::{message, WeakAddr};

use crate::{connection::Connection, session::Session};

#[message]
pub enum Command {
    AssociateConnection {
        connection: WeakAddr<Connection>,
        credentials: Credentials,
    },
}

#[message]
pub struct SessionAssociated {
    pub session: WeakAddr<Session>,
}

#[message]
#[derive(Clone, Copy, Debug)]
pub struct Gc;
