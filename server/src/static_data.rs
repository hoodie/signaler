//! Placeholder for a database

use serde::{Deserialize, Serialize};

use std::{collections::HashMap, fs};

use crate::user_management::{UserId, UserProfile};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct StaticUserDatabase {
    pub credentials: HashMap<UserId, String>,
    pub profiles: HashMap<UserId, UserProfile>,
}

impl StaticUserDatabase {
    pub fn load() -> Self {
        serde_json::from_str(&fs::read_to_string("./config/test_users.json").unwrap()).unwrap()
    }
}
