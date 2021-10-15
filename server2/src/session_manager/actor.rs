use async_trait::async_trait;
use tracing::log;
use xactor::{Actor, Context, Handler};

use super::{command::*, SessionManager};

#[async_trait]
impl Actor for SessionManager {
    async fn started(&mut self, _ctx: &mut xactor::Context<Self>) -> xactor::Result<()> {
        log::trace!("starting SessionManager");

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
                }
            }
        }
    }
}
impl xactor::Service for SessionManager {}
