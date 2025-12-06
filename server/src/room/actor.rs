use hannibal::{Actor, Handler};
use protocol::ChatMessage;
use signaler_protocol as protocol;
use tracing::log;
use uuid::Uuid;

use super::{
    command::{ChatRoomCommand, Command},
    Room,
};

impl Actor for Room {
    async fn started(&mut self, _ctx: &mut hannibal::Context<Self>) -> hannibal::DynResult<()> {
        // log::info!("starting Room {:?}", ctx.actor_id());
        Ok(())
    }
    async fn stopped(&mut self, _ctx: &mut hannibal::Context<Self>) {
        log::trace!("shutting down Room");
    }
}

impl Handler<Command> for Room {
    async fn handle(&mut self, ctx: &mut hannibal::Context<Self>, cmd: Command) {
        log::trace!("received command {:?}", cmd);
        match cmd {
            Command::AddParticipant { participant } => self.add_participant(participant, ctx),
        }
    }
}

impl Handler<ChatRoomCommand> for Room {
    async fn handle(&mut self, ctx: &mut hannibal::Context<Self>, cmd: ChatRoomCommand) {
        log::trace!("received command {:?}", cmd);
        match cmd.command {
            protocol::ChatRoomCommand::Leave => {
                log::trace!("received leave, but from whom?");
                todo!()
            }
            protocol::ChatRoomCommand::Message { content } => {
                log::trace!("forwarding message {content:?}");
                self.forward_to_participants(
                    ChatMessage {
                        content,
                        sender: cmd.session_id.into(),
                        sent: chrono::Utc::now(),
                        uuid: Uuid::new_v4(),
                    },
                    ctx,
                )
            }
            protocol::ChatRoomCommand::ListParticipants => todo!(),
        }
    }
}
