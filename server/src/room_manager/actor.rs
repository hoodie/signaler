use std::time::Duration;

use async_trait::async_trait;
use tracing::log;
use xactor::{Actor, Handler};

use crate::metrics::MetricsService;

use super::{command::*, RoomManager};

#[async_trait]
impl Actor for RoomManager {
    async fn started(&mut self, ctx: &mut xactor::Context<Self>) -> xactor::Result<()> {
        log::trace!("starting");
        if let Some(gauge) = MetricsService::get_gauge("open_rooms", "open rooms").await? {
            log::debug!("instantiated room gauge");
            self.open_rooms = Some(gauge);
        }
        ctx.send_interval(Gc, Duration::from_secs(5));

        Ok(())
    }
    async fn stopped(&mut self, _ctx: &mut xactor::Context<Self>) {
        log::trace!("shutting down");
    }
}

#[async_trait::async_trait]
impl Handler<Command> for RoomManager {
    async fn handle(&mut self, _ctx: &mut xactor::Context<Self>, cmd: Command) {
        log::trace!("received command {:?}", cmd);
        match cmd {
            Command::JoinRoom { room_id, participant } => self.join_room(&room_id, participant).await,
        }
    }
}

#[async_trait::async_trait]
impl Handler<Gc> for RoomManager {
    async fn handle(&mut self, ctx: &mut xactor::Context<Self>, _: Gc) {
        self.gc(ctx);
    }
}

impl xactor::Service for RoomManager {}