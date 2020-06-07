#![allow(unused_imports)]
use actix::{prelude::*, WeakAddr};
#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

use crate::{
    presence::{
        command::{AuthToken, AuthenticationRequest},
        message::AuthResponse,
        Credentials, SimplePresenceService,
    },
    session::ClientSession,
};

use std::{collections::HashMap, time::Duration};
use uuid::Uuid;

mod default;
use default::DefaultSessionManager;

pub mod command {
    use signaler_protocol::*;

    use super::*;
    use crate::socket_connection::SocketConnection;

    use actix::prelude::*;
    #[allow(unused_imports)]
    use log::{debug, error, info, trace, warn};
    #[derive(Message)]
    #[rtype(result = "()")]
    pub struct GetSession {
        pub credentials: Credentials,
        pub connection: WeakAddr<SocketConnection>
    }

    impl Handler<GetSession> for SessionManagerService {
        type Result = ();

        fn handle(&mut self, command: GetSession, ctx: &mut Self::Context) -> Self::Result {
            let GetSession { credentials, connection } = command;

            self.get_session(&credentials, connection, ctx);
        }
    }
}

pub type SessionManagerService = DefaultSessionManager;

impl Actor for SessionManagerService {
    type Context = Context<Self>;
    fn started(&mut self, ctx: &mut Self::Context) {
        debug!("SessionManager started");
        ctx.run_interval(Duration::from_millis(30_000), Self::gc);
    }
}

impl SystemService for SessionManagerService {}
impl Supervised for SessionManagerService {}
