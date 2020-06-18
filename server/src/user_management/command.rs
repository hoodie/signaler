use actix::prelude::*;

use super::*;

#[derive(Message)]
#[rtype(result = "Option<UserProfile>")]
pub struct WhoIsRequest {
    pub user_id: UserId,
}

impl Handler<WhoIsRequest> for UserService {
    type Result = MessageResult<WhoIsRequest>;

    fn handle(&mut self, request: WhoIsRequest, _ctx: &mut Self::Context) -> Self::Result {
        log::info!("received WhoIsRequest");

        let WhoIsRequest { user_id } = request;

        MessageResult(self.who_is(&user_id))
    }
}
