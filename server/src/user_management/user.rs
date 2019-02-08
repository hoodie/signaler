use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, actix::Message, Serialize, Deserialize)]
pub struct UserProfile {
    pub full_name: String,
}

