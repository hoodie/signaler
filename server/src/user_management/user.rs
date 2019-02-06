#[derive(Debug, actix::Message)]
pub struct UserProfile {
    pub user_name: String,
}

