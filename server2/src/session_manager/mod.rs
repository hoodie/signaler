use xactor::Addr;

#[derive(Debug, Default)]
pub struct DefaultSessionManager {
    sessions: HashMap<SessionKey, Addr<Session>>,
}

impl SessionManager {
    pub fn get_session(
        &mut self,
        creds: &Credentials,
        connection: WeakRecipient<SessionConnected>,
        ctx: &mut Context<Self>,
    ) {
        self.create_session(creds, connection, ctx)
    }

}
