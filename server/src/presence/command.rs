use actix::prelude::*;

use super::*;

/// Token returned after successful authentication
///
/// Use this to make requests that require authentication. Should timeout.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct AuthToken(Uuid);

impl AuthToken {
    pub fn new() -> Self {
        Default::default()
    }
}

impl Default for AuthToken {
    fn default() -> Self {
        Self(Uuid::new_v4())
    }
}

/// Message expected by PresenceService to add SessionId
#[derive(Message, Debug, Clone)]
#[rtype(result = "Option<message::SimpleAuthResponse>")]
pub struct AuthenticationRequest {
    pub credentials: Credentials,
    // TODO pub connection_id: Uuid,
}

/// implementation docs
impl Handler<AuthenticationRequest> for PresenceService<Credentials, AuthToken> {
    type Result = MessageResult<AuthenticationRequest>;

    fn handle(&mut self, request: AuthenticationRequest, _ctx: &mut Self::Context) -> Self::Result {
        log::info!("received AuthenticationRequest");
        let AuthenticationRequest {
            credentials,
            // connection_id,
        } = request;
        // FIXME: connection_id
        MessageResult(self.associate_user(&credentials, &uuid::Uuid::new_v4()))
    }
}

/// Message expected by PresenceService to add SessionId
#[derive(Message, Debug)]
#[rtype(result = "bool")]
pub struct ValidateRequest {
    pub token: AuthToken,
}

impl Handler<ValidateRequest> for PresenceService<Credentials, AuthToken> {
    type Result = MessageResult<ValidateRequest>;
    fn handle(&mut self, request: ValidateRequest, _ctx: &mut Self::Context) -> Self::Result {
        let ValidateRequest { token } = request;
        MessageResult(self.still_valid(&token))
    }
}
