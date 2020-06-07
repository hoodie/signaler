use actix::{prelude::*, WeakAddr};
#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

use std::time::Duration;

mod default;
use default::DefaultSessionManager;

pub mod command;

pub type SessionManagerService = DefaultSessionManager;

impl Actor for SessionManagerService {
    type Context = Context<Self>;
    fn started(&mut self, ctx: &mut Self::Context) {
        debug!("SessionManager started");
        ctx.run_interval(Duration::from_millis(30_000), Self::gc);
    }
}

impl SystemService for SessionManagerService {}
impl Supervised for SessionManagerService {}
