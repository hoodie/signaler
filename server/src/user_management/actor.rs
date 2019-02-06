use super::*;
use super::user::UserProfile;

use actix::prelude::*;

impl Actor for UserManagement<UserProfile> {
    type Context = Context<Self>;
}

impl SystemService for UserManagement<UserProfile> {}
impl Supervised for UserManagement<UserProfile>{}

#[derive(Message)]
#[rtype(result = "Option<UserProfile>")]
pub struct AuthenticationRequest {
    pub user: UserId,
    pub session_id: SessionId,
    pub password: String,
}

impl Handler<AuthenticationRequest> for UserManagement<UserProfile> {
    type Result = MessageResult<AuthenticationRequest>;

    fn handle(&mut self, request: AuthenticationRequest, _ctx: &mut Self::Context) -> Self::Result {
        info!("received AuthenticationRequest");

        let AuthenticationRequest {user, session_id, password} = request;
        
        MessageResult(self.associate_user(&user, &session_id, &password))
    }
}

