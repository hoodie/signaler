//! Placeholder for a database

use serde::{Deserialize, Serialize};

use std::collections::HashMap;

use crate::user_management::{UserId, UserProfile};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct StaticUserDatabase {
    pub credentials: HashMap<UserId, String>,
    pub profiles: HashMap<UserId, UserProfile>,
}

impl StaticUserDatabase {
    pub fn load() -> Self {
        serde_json::from_str(include_str!("../test_users.json")).unwrap()
    }
}
