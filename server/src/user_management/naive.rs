#[allow(unused_imports)]
use log::{debug, error, info};

use crate::static_data::StaticUserDatabase;

use super::*;

/// Naive implementation of a `UserService`
#[derive(Debug, Default)]
pub struct NaiveUserManager {
    pub user_database: StaticUserDatabase,
}

impl NaiveUserManager {
    pub fn naive() -> UserService {
        let manager = NaiveUserManager {
            user_database: StaticUserDatabase::load(),
        };
        debug!("new NaiveUserManager {:?}", manager);
        UserManager::new(Box::new(manager))
    }
}
