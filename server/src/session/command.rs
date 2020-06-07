use actix::prelude::*;
use actix::WeakAddr;

#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};
use signaler_protocol::*;

use super::ClientSession;
use crate::{
    room::{command::UpdateParticipant, message::RoomToSession, DefaultRoom},
    socket_connection::SocketConnection,
};

#[derive(Message, Debug)]
// #[rtype(result = "Option<UserProfile>")]
#[rtype(result = "()")]
pub struct ProvideProfile<T: Actor> {
    pub room_addr: WeakAddr<T>,
}

impl Handler<ProvideProfile<DefaultRoom>> for ClientSession {
    type Result = MessageResult<ProvideProfile<DefaultRoom>>;

    fn handle(&mut self, p: ProvideProfile<DefaultRoom>, ctx: &mut Context<Self>) -> Self::Result {
        if let Some(profile) = self.profile.clone() {
            if let Some(addr) = p.room_addr.upgrade() {
                addr.send(UpdateParticipant {
                    session_id: self.session_id,
                    profile,
                })
                .into_actor(self)
                .then(|_, _, _| fut::ready(()))
                .spawn(ctx);
            }
        } else {
            warn!("{:?} was asked for profile, but didn't have one", self.session_id);
        }
        MessageResult(())
    }
}

#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct SessionCommand(pub signaler_protocol::SessionCommand);

impl Handler<SessionCommand> for ClientSession {
    type Result = ();

    fn handle(&mut self, SessionCommand(cmd): SessionCommand, ctx: &mut Context<Self>) -> Self::Result {
        self.dispatch_incoming_message(cmd, ctx)
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct RegisterConnection {
    pub connection: WeakAddr<SocketConnection>,
}

impl Handler<RegisterConnection> for ClientSession {
    type Result = ();

    fn handle(
        &mut self,
        RegisterConnection { connection }: RegisterConnection,
        _ctx: &mut Context<Self>,
    ) -> Self::Result {
        debug!("registered connection");
        self.connection.replace(connection);
    }
}

impl Handler<RoomToSession> for ClientSession {
    type Result = ();

    fn handle(&mut self, msg: RoomToSession, ctx: &mut Self::Context) -> Self::Result {
        debug!("received message from Room");
        match msg {
            RoomToSession::Joined(id, addr) => {
                info!("successfully joined room {:?}", id);
                self.rooms.insert(id, addr);
                self.list_my_rooms(ctx);
            }

            RoomToSession::Left { room } => {
                info!("successfully left room {:?}", room);
                if let Some(room) = self.rooms.remove(&room) {
                    debug!("removed room {:?} from {:?}", room, self.session_id);
                } else {
                    error!(
                        "remove from room {:?}, but had no reference ({:?})",
                        room, self.session_id
                    );
                }
                self.list_my_rooms(ctx);
            }

            RoomToSession::ChatMessage { room, message } => {
                self.send_message(SessionMessage::Message { message, room }, ctx)
            }

            RoomToSession::RoomState { room, roster } => {
                debug!("forwarding participants for room: {:?}\n{:#?}", room, roster);
                self.send_message(
                    SessionMessage::RoomParticipants {
                        room,
                        participants: roster,
                    },
                    ctx,
                )
            }

            RoomToSession::RoomEvent { room, event } => {
                debug!("forwarding event from room: {:?}\n{:#?}", room, event);
                self.send_message(SessionMessage::RoomEvent { room, event }, ctx)
            }

            RoomToSession::History { room, mut messages } => {
                // TODO: Self::send_history
                for message in messages.drain(..) {
                    self.send_message(
                        SessionMessage::Message {
                            message,
                            room: room.clone(),
                        },
                        ctx,
                    )
                }
            }

            RoomToSession::JoinDeclined { room } => self.send_message(
                SessionMessage::Error {
                    message: format!("unable to join room {}", room),
                },
                ctx,
            ),
        }
    }
}
