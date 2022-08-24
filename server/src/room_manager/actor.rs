use std::time::Duration;

use async_trait::async_trait;
use hannibal::{Actor, Handler};
use tracing::log;

use crate::metrics::MetricsService;

use super::{command::*, RoomManager};

#[async_trait]
impl Actor for RoomManager {
    const NAME: &'static str = module_path!();

    async fn started(&mut self, ctx: &mut hannibal::Context<Self>) -> hannibal::Result<()> {
        log::trace!("starting");
        if let Some(gauge) = MetricsService::get_gauge("open_rooms", "open rooms").await? {
            log::debug!("instantiated room gauge");
            self.open_rooms = Some(gauge);
        }
        ctx.send_interval(Gc, Duration::from_secs(5));

        Ok(())
    }
    async fn stopped(&mut self, _ctx: &mut hannibal::Context<Self>) {
        log::trace!("shutting down");
    }
}

#[async_trait::async_trait]
impl Handler<Command> for RoomManager {
    #[tracing::instrument(level = tracing::Level::INFO, skip_all)]
    async fn handle(&mut self, _ctx: &mut hannibal::Context<Self>, cmd: Command) {
        log::trace!("received command {:?}", cmd);
        match cmd {
            Command::JoinRoom { room_id, participant } => self.join_room(&room_id, participant).await,
        }
    }
}

#[async_trait::async_trait]
impl Handler<Gc> for RoomManager {
    async fn handle(&mut self, ctx: &mut hannibal::Context<Self>, _: Gc) {
        self.gc(ctx);
    }
}

impl hannibal::Service for RoomManager {}
