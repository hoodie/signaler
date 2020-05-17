use serde::{Deserialize, Serialize};
use std::ops::Deref;

#[derive(Clone, Debug, actix::Message, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[rtype(result = "()")]
pub struct UserProfile(signaler_protocol::UserProfile);

impl From<signaler_protocol::UserProfile> for UserProfile {
    fn from(user_profile: signaler_protocol::UserProfile) -> UserProfile {
        UserProfile(user_profile)
    }
}

impl Into<signaler_protocol::UserProfile> for UserProfile {
    fn into(self) -> signaler_protocol::UserProfile {
        self.0
    }
}

impl Deref for UserProfile {
    type Target = signaler_protocol::UserProfile;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

