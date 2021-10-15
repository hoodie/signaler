use signaler_protocol::Credentials;
use xactor::{message, Sender, WeakAddr};

use crate::session::Session;

#[message]
pub enum Command {
    AssociateConnection {
        connection: Sender<SessionAssociated>,
        credentials: Credentials,
    },
}

#[message]
pub struct SessionAssociated {
    pub session: WeakAddr<Session>,
}
