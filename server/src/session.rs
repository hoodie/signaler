//! # Signaling Session
//!
//! One session per participant

// TODO: how to timeout sessions?

use actix::prelude::*;
use actix::WeakAddr;
use actix_web_actors::ws::{self, WebsocketContext};
use log::*;
use uuid::Uuid;

use std::collections::HashMap;
    use std::fmt;

use crate::protocol::*;
use crate::presence::{AuthToken, AuthResponse, UsernamePassword, SimplePresenceService, AuthenticationRequest};
use crate::room::{self, DefaultRoom, RoomId, Participant};
use crate::room_manager::{self, RoomManagerService};
use crate::user_management::UserProfile;

pub type SessionId = Uuid;

pub struct ClientSession {
    session_id: SessionId,
    token: Option<AuthToken>,
    profile: Option<UserProfile>,
    rooms: HashMap<RoomId, WeakAddr<DefaultRoom>>,
}

impl fmt::Debug for ClientSession {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ClientSession {{ {} }}", self.session_id)
    }
}

impl ClientSession {

    /// parses raw string and passes it to `dispatch_incoming_message` or replies with error
    fn handle_incoming_message(&self, raw_msg: &str, ctx: &mut WebsocketContext<Self>) {
        // trace!("handle message: {:?}", raw_msg);
        let parsed: Result<SessionCommand, _> = serde_json::from_str(raw_msg);
        if let Ok(msg) = parsed {
            trace!("parsed ok\n{}\n{:?}", raw_msg, msg);
            self.dispatch_incoming_message(msg, ctx)

        } else {
            warn!("cannot parse: {}", raw_msg);
            debug!("suggestions:\n{}", SessionCommand::suggestions())
        }
    }

    /// react to client messages
    fn dispatch_incoming_message(&self, msg: SessionCommand, ctx: &mut WebsocketContext<Self>) {
        match msg {
            SessionCommand::Authenticate { credentials } => {
                trace!("received credentials {:?}", credentials);
                self.authenticate(credentials, ctx);
            },

            SessionCommand::ListRooms => {
                trace!("received ListRooms signal");
                self.list_rooms(ctx);
            },

            SessionCommand::Join{ room } => {
                trace!("requesting to Join {}", room);
                self.join(&room, ctx);
            },

            SessionCommand::ListMyRooms => {
                trace!("received ListMyRooms signal");
                self.list_my_rooms(ctx);
            },

            SessionCommand::Message{ message, room } => {
                trace!("received message to forward");
                self.forward_message( message, &room, ctx);
            },

            SessionCommand::ShutDown => {
                trace!("received shut down signal");
                System::current().stop();
            },
        }
    }

    /// send message to client
    fn send_message(message: SessionMessage, ctx: &mut WebsocketContext<Self>) {
        ctx.text(message.into_json())
    }

    fn list_rooms(&self, ctx: &mut WebsocketContext<Self>) {
        let msg = room_manager::command::ListRooms;

        RoomManagerService::from_registry()
            .send(msg)
            .into_actor(self)
            .then(|rooms, _, ctx| {
                debug!("list request answered: {:?}", rooms);
                match rooms {
                    Ok(rooms) => Self::send_message(SessionMessage::RoomList{rooms}, ctx),
                    Err(error) => ctx.text(SessionMessage::err(format!("{:?}", error)).into_json())
                }
                fut::ok(())
            })
            .spawn(ctx);
    }

    fn list_my_rooms(&self, ctx: &mut WebsocketContext<Self>) {
        let rooms = self.rooms.keys().cloned().collect::<Vec<String>>();
        debug!("my list request answered: {:?}", rooms);
        Self::send_message(SessionMessage::MyRoomList{rooms}, ctx);
    }

    fn join(&self, room: &str, ctx: &mut WebsocketContext<Self>) {
        if let Some(token) = dbg!(self.token) {
            let msg = room_manager::command::JoinRoom {
                room: room.into(),
                participant: Participant {
                    session_id: self.session_id,
                    addr: ctx.address().downgrade()
                },
                // return_addr: ctx.address().recipient(),
                token
            };

            RoomManagerService::from_registry()
                .send(msg)
                .into_actor(self)
                .then(|_, _, _| {
                    debug!("join request forwarded to room successfully");
                    fut::ok(())
                })
                .spawn(ctx);
        } else {
            warn!("can't join room, no authentication token")
        }

    }

    fn leave_all_rooms(&mut self, _ctx: &mut WebsocketContext<Self>) {
        use room::command::RemoveParticipant;
        let rooms_to_leave: HashMap<String, WeakAddr<DefaultRoom>> = self.rooms.drain().collect();
        for (name, addr) in dbg!(rooms_to_leave) {
            trace!("sending RemoveParticipant to {:?} (waiting)", name);
            addr.upgrade().unwrap()
                .send(RemoveParticipant { session_id: self.session_id })
                .timeout(std::time::Duration::new(1, 0))
                .wait().unwrap();
        }
        trace!("all rooms left");
    }

    fn authenticate(&self, credentials: UsernamePassword, ctx: &mut WebsocketContext<Self>) {
        trace!("session starts authentication process");
        let msg = AuthenticationRequest {
            credentials,
            session_id: self.session_id
        };
        SimplePresenceService::from_registry()
            .send(msg)
            .into_actor(self)
            .then(|profile, client_session, ctx| {
                debug!("userProfile {:?}", profile);
                match profile {
                    Ok(Some(AuthResponse {token, profile})) => {
                        info!("authenticated {:?}", token);
                        client_session.token = Some(token);
                        client_session.profile = profile.clone();
                        Self::send_message(SessionMessage::Authenticated, ctx);
                        if let Some(profile) = profile {
                            Self::send_message(SessionMessage::Profile { profile }, ctx);
                        }
                    }
                    Ok(None) => Self::send_message(SessionMessage::Error{ message: String::from("unabled to login")}, ctx),
                    Err(error) => ctx.text(SessionMessage::err(format!("{:?}", error)).into_json())
                }
                fut::ok(())
            })
            .spawn(ctx);
    }

    fn forward_message(&self, content: String, room: &str, ctx: &mut WebsocketContext<Self>) {
        let msg = room::command::Forward {
            message: ChatMessage::new(content, self.session_id),
            sender: self.session_id,
        };

        if let Some(room) = dbg!(self.rooms.get(room)) {
            room.upgrade().unwrap().send(msg)
                .into_actor(self)
                .then(|resp, _, ctx| {
                    debug!("message forwared -> {:?}", resp);
                    match resp {
                        Ok(Err(message)) => {
                            debug!("message rejected {:?}", message);
                            Self::send_message(SessionMessage::Error{message}, ctx)
                        },
                        Ok(mr) => {
                            debug!("ok after all -> {:?}", mr);
                        }
                        Err(error) => Self::send_message(SessionMessage::Error{ message: error.to_string()}, ctx)
                    }
                    fut::ok(())
                })
                .spawn(ctx);
        } else {
            error!("no such room {:?}", room);
        }
    }
}

impl Default for ClientSession {
    fn default() -> Self {
        Self {
            session_id: Uuid::new_v4(),
            token: None,
            profile: None,
            rooms: Default::default()
        }
    }
}

impl Actor for ClientSession {
    type Context = WebsocketContext<Self>;
    fn started(&mut self, ctx: &mut Self::Context) {
        info!("ClientSession started {:?}", self.session_id);
        ClientSession::send_message(SessionMessage::Welcome{ session: SessionDescription { session_id: self.session_id } }, ctx);
    }

    fn stopping(&mut self, ctx: &mut Self::Context) -> Running {
        self.leave_all_rooms(ctx);

        Running::Stop
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        debug!("ClientSsession stopped: {}", self.session_id);
    }
}

impl StreamHandler<ws::Message, ws::ProtocolError> for ClientSession {
    fn handle(&mut self, msg: ws::Message, ctx: &mut Self::Context) {
        match msg {
            ws::Message::Ping(msg) => ctx.pong(&msg),
            ws::Message::Text(text) => {
                self.handle_incoming_message(&text, ctx);
            },
            ws::Message::Close(reason) => {
                info!("websocket was closed {:?}", reason);
                ctx.stop();
            },
            _ => (), // Pong, Nop, Binary
        }
    }
}

use crate::room::message::RoomToSession;
impl Handler<RoomToSession> for ClientSession {
    type Result = ();

    fn handle(&mut self, msg: RoomToSession, ctx: &mut Self::Context) -> Self::Result {
        debug!("received message from Room");
        match msg {
            RoomToSession::Joined(id, addr) => {
                info!("successfully joined room {:?}", id);
                self.rooms.insert(id, addr);
                self.list_my_rooms(ctx);
            },

            RoomToSession::ChatMessage{ room, message } => {
                Self::send_message(SessionMessage::Message{message, room}, ctx)
            },

            RoomToSession::Participants{ room, participants } => {
                debug!("forwarding participants for room: {:?}\n{:#?}", room, participants);
                Self::send_message(SessionMessage::RoomParticipants{ room, participants }, ctx)
            },

            RoomToSession::History{ room, mut messages } => {
                // TODO: Self::send_history
                for message in messages.drain(..) {
                    Self::send_message(SessionMessage::Message{ message, room: room.clone() }, ctx)
                }
            },

            RoomToSession::JoinDeclined{ room } => {
                Self::send_message(SessionMessage::Error{message: format!("unable to join room {}", room) }, ctx)
            }
        }
    }
}

pub mod command {
    use actix::prelude::*;
    use actix_web_actors::ws::WebsocketContext;
    #[allow(unused_imports)]
    use log::{info, error, debug, warn, trace};

    use super::ClientSession;
    use crate::user_management::UserProfile;

    #[derive(Message)]
    #[rtype(result = "Option<UserProfile>")]
    pub struct ProvideProfile;

    impl Handler<ProvideProfile> for ClientSession {
        type Result = MessageResult<ProvideProfile>;

        fn handle(&mut self, command: ProvideProfile, ctx: &mut WebsocketContext<Self>) -> Self::Result {
            MessageResult(self.profile.clone())
        }
    }
}