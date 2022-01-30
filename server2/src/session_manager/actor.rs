use std::time::Duration;

use async_trait::async_trait;
use tracing::log;
use xactor::{Actor, Context, Handler};

use crate::metrics::MetricsService;

use super::{command::*, SessionManager};

#[async_trait]
impl Actor for SessionManager {
    async fn started(&mut self, ctx: &mut xactor::Context<Self>) -> xactor::Result<()> {
        log::trace!("starting SessionManager");
        if let Some(gauge) = MetricsService::get_gauge("open_sessions", "open session").await? {
            log::debug!("instantiated session gauge");
            self.open_sessions = Some(gauge);
        }
        ctx.send_interval(Gc, Duration::from_secs(5));

        Ok(())
    }
    async fn stopped(&mut self, _ctx: &mut xactor::Context<Self>) {
        log::trace!("shutting down SessionManager");
    }
}

#[async_trait]
impl Handler<Command> for SessionManager {
    async fn handle(&mut self, _ctx: &mut Context<Self>, cmd: Command) {
        match cmd {
            Command::AssociateConnection {
                credentials,
                connection,
            } => {
                if let Err(error) = self.create_session(&credentials, connection).await {
                    log::error!("failed to associate {}", error);
                } else if let Some(gauge) = self.open_sessions.as_ref() {
                    gauge.inc();
                    log::trace!("increasing sessions count {:?}", gauge.get());
                }
            }
        }
    }
}

#[async_trait::async_trait]
impl Handler<Gc> for SessionManager {
    async fn handle(&mut self, ctx: &mut xactor::Context<Self>, _: Gc) {
        self.gc(ctx);
    }
}

impl xactor::Service for SessionManager {}
