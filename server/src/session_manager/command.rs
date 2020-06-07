use signaler_protocol::*;

use super::*;
use crate::socket_connection::SocketConnection;

#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};
#[derive(Message)]
#[rtype(result = "()")]
pub struct GetSession {
    pub credentials: Credentials,
    pub connection: WeakAddr<SocketConnection>,
}

impl Handler<GetSession> for SessionManagerService {
    type Result = ();

    fn handle(&mut self, command: GetSession, ctx: &mut Self::Context) -> Self::Result {
        let GetSession {
            credentials,
            connection,
        } = command;

        self.get_session(&credentials, connection, ctx);
    }
}
