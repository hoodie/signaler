// h
use actix::{prelude::*, WeakAddr};
#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};
use uuid::Uuid;

use crate::{
    presence::{
        command::{AuthToken, AuthenticationRequest},
        message::AuthResponse,
        Credentials, SimplePresenceService,
    },
    session::ClientSession,
    socket_connection::SocketConnection,
};

use std::{collections::HashMap, time::Duration};

use super::SessionManagerService;

#[derive(Debug, Hash, Eq, PartialEq)]
enum SessionKey {
    Token(AuthToken),
    Name(String),
}

#[derive(Debug, Default)]
pub struct DefaultSessionManager {
    sessions: HashMap<SessionKey, Addr<ClientSession>>,
}

impl DefaultSessionManager {
    pub fn get_session(
        &mut self,
        creds: &Credentials,
        connection: WeakAddr<SocketConnection>,
        ctx: &mut Context<Self>,
    ) {
        self.create_session(creds, connection, ctx)
    }

    /// Create a new Session
    pub fn create_session(
        &mut self,
        creds: &Credentials,
        connection: WeakAddr<SocketConnection>,
        ctx: &mut Context<Self>,
    ) {
        trace!("session starts authentication process");

        let msg = AuthenticationRequest {
            credentials: creds.clone(),
        };

        SimplePresenceService::from_registry()
            .send(msg)
            .into_actor(self)
            .then(move |profile, slf, ctx| {
                debug!("userProfile {:?}", profile);
                match profile {
                    Ok(Some(AuthResponse { token, profile })) => {
                        if let Some(connection) = connection.upgrade() {
                            let session = ClientSession::from_token_and_profile(token, profile).start();
                            let weak_session = session.downgrade();
                            slf.sessions.insert(SessionKey::Token(token), session);
                            connection
                                .send(crate::socket_connection::command::SessionConnected { session: weak_session })
                                .into_actor(slf)
                                .then(|_, _, _| fut::ready(()))
                                .spawn(ctx);
                        } else {
                            warn!("session can be created but connection was dead")
                        }
                    }
                    Ok(None) => {
                        // TODO: don't need to bother the authservice here
                        // let session = ClientSession { ..Default::default() };
                        // slf.sessions.insert(SessionKey::token, session);
                    }
                    Err(error) => {
                        // ctx.text(SessionMessage::err(format!("{:?}", error)).into_json()),
                    }
                }
                fut::ready(())
            })
            .spawn(ctx);
    }

    /// Check each session's connections and toss them
    pub fn gc(&mut self, _ctx: &mut Context<Self>) {
        // let (live_session, dead_sessions) = self.sessions.drain().partition(|(_, s)| s.has_connection());
        // self.sessions = live_session;

        // for (id, _session) in &dead_sessions {
        //     debug!("{} is dead", id)
        //     // ctx.send
        // }
    }
}
