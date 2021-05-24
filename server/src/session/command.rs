use actix::{prelude::*, WeakAddr};

use signaler_protocol::*;

use super::ClientSession;
use crate::{
    room::{self, message::RoomToSession, DefaultRoom},
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

    fn handle(&mut self, p: ProvideProfile<DefaultRoom>, _ctx: &mut Context<Self>) -> Self::Result {
        if let Some(profile) = self.profile.clone() {
            if let Some(addr) = p.room_addr.upgrade() {
                if let Err(error) = addr.try_send(room::Command::UpdateParticipant {
                    session_id: self.session_id,
                    profile,
                }) {
                    log::error!("{}", error)
                }
            }
        } else {
            log::warn!("{:?} was asked for profile, but didn't have one", self.session_id);
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
        log::debug!("registered connection");
        self.connection.replace(connection);
    }
}

impl Handler<RoomToSession> for ClientSession {
    type Result = ();

    fn handle(&mut self, msg: RoomToSession, ctx: &mut Self::Context) -> Self::Result {
        log::debug!("received message from Room");
        match msg {
            RoomToSession::Joined(id, addr) => {
                log::info!("successfully joined room {:?}", id);
                self.rooms.insert(id, addr);
                self.list_my_rooms(ctx);
            }

            RoomToSession::Left { room } => {
                log::info!("successfully left room {:?}", room);
                if let Some(room) = self.rooms.remove(&room) {
                    log::debug!("removed room {:?} from {:?}", room, self.session_id);
                } else {
                    log::error!(
                        "remove from room {:?}, but had no reference ({:?})",
                        room,
                        self.session_id
                    );
                }
                self.list_my_rooms(ctx);
            }

            RoomToSession::ChatMessage { room, message } => {
                self.send_message(SessionMessage::Message { message, room }, ctx)
            }

            RoomToSession::RoomState { room, roster } => {
                log::debug!("forwarding participants for room: {:?}\n{:#?}", room, roster);
                self.send_message(
                    SessionMessage::RoomParticipants {
                        room,
                        participants: roster,
                    },
                    ctx,
                )
            }

            RoomToSession::RoomEvent { room, event } => {
                log::debug!("forwarding event from room: {:?}\n{:#?}", room, event);
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
