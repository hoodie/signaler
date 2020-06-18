use crate::static_data::StaticUserDatabase;

use super::*;

/// Naive implementation of a `UserService`
#[derive(Debug, Default)]
pub struct NaiveUserManager {
    user_database: StaticUserDatabase,
}

impl NaiveUserManager {
    pub fn naive() -> UserService {
        let manager = NaiveUserManager {
            user_database: StaticUserDatabase::load(),
        };
        log::debug!("new NaiveUserManager {:?}", manager);
        UserService::new(Box::new(manager))
    }
}

impl UserManaging for NaiveUserManager {
    type UserProfile = UserProfile;

    fn who_is(&self, user_id: &str) -> Option<UserProfile> {
        if let Some(profile) = self.user_database.profiles.get(user_id) {
            log::info!("found profile {:?}", user_id);
            Some(profile.clone())
        } else {
            log::error!("found user but not profile {:?}", user_id);
            None
        }
    }
    fn update(&mut self) {
        self.user_database = StaticUserDatabase::load();
        log::trace!("usermanager updated {:?}", self.user_database);
    }
}
