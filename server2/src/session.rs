use uuid::Uuid;
use xactor::{Actor, Handler};

pub struct Session {
    pub session_id: Uuid,
}

impl Default for Session {
    fn default() -> Self {
        Session {
            session_id: Uuid::new_v4(),
        }
    }
}

#[async_trait::async_trait]
impl Actor for Session {
    async fn started(&mut self, ctx: &mut xactor::Context<Self>) -> xactor::Result<()> {
        log::info!("starting session on actor {:?}", ctx.actor_id());
        Ok(())
    }
}

#[xactor::message]
struct Foo;

#[async_trait::async_trait]
impl Handler<Foo> for Session {
    async fn handle(&mut self, _ctx: &mut xactor::Context<Self>, _msg: Foo) {
        log::info!("received a Foo");
    }
}
