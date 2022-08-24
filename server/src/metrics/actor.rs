use async_trait::async_trait;
use hannibal::{Actor, Context, Handler, Service};
use prometheus::{IntGauge, Registry};
use tracing::log;

use super::{command::*, MetricsService};

#[async_trait]
impl Actor for MetricsService {
    const NAME: &'static str = module_path!();

    async fn started(&mut self, _ctx: &mut hannibal::Context<Self>) -> hannibal::Result<()> {
        log::trace!("starting MetricsService");

        Ok(())
    }
    async fn stopped(&mut self, _ctx: &mut hannibal::Context<Self>) {
        log::trace!("shutting down MetricsService");
    }
}

impl Service for MetricsService {}

#[async_trait]
impl Handler<GetRegistry> for MetricsService {
    async fn handle(&mut self, _ctx: &mut Context<Self>, _cmd: GetRegistry) -> Registry {
        log::trace!("passing out registry");
        self.registry.clone()
    }
}

#[async_trait]
impl Handler<AddGauge> for MetricsService {
    async fn handle(&mut self, _ctx: &mut Context<Self>, cmd: AddGauge) -> Option<IntGauge> {
        self.add_gauge(&cmd.name, &cmd.help)
    }
}
