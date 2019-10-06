use actix::prelude::*;
use log::*;
use serde::{Deserialize, Serialize};

use crate::static_data::StaticUserDatabase;

use std::ops::Deref;

pub type UserId = String;

#[derive(Clone, Debug, actix::Message, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserProfile (signaler_protocol::UserProfile);

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

pub trait UserManaging {
    type UserProfile;
    fn who_is(&self, user_id: &str) -> Option<Self::UserProfile>;
}

pub struct UserManager<P> {
    inner: Box<dyn UserManaging<UserProfile = P>>,
}

impl<P> UserManager<P> {
    pub fn new(implementation: Box<dyn UserManaging<UserProfile=P>>) -> Self {
        Self { inner: implementation }
    }
}

impl<P> UserManaging for UserManager<P> {
    type UserProfile = P;
    fn who_is(&self, user_id: &str) -> Option<Self::UserProfile> {
        self.inner.who_is(user_id)
    }
}

impl Actor for UserManager<UserProfile> {
    type Context = Context<Self>;
}

type UserService = UserManager<UserProfile>;
impl SystemService for UserService {}
impl Supervised for UserService {}

#[derive(Message)]
#[rtype(result = "Option<UserProfile>")]
pub struct WhoIsRequest {
    pub user_id: UserId,
}

impl Handler<WhoIsRequest> for UserService {
    type Result = MessageResult<WhoIsRequest>;

    fn handle(&mut self, request: WhoIsRequest, _ctx: &mut Self::Context) -> Self::Result {
        info!("received WhoIsRequest");

        let WhoIsRequest {user_id} = request;

        MessageResult(self.who_is(&user_id))
    }
}


impl Default for UserService {
    fn default() -> Self {
        NaiveUserManager::naive()
    }
}

#[derive(Debug, Default)]
pub struct NaiveUserManager {
    user_database: StaticUserDatabase
}

impl NaiveUserManager {
    pub fn naive() -> UserService {
        let manager = NaiveUserManager {
            user_database: StaticUserDatabase::load()
        };
        debug!("new NaiveUserManager {:?}", manager);
        UserManager::new(Box::new(manager))
    }
}

impl UserManaging for NaiveUserManager {
    type UserProfile = UserProfile;

    fn who_is(&self, user_id: &str) -> Option<UserProfile> {
        if let Some(profile) = self.user_database.profiles.get(user_id) {
            info!("found profile {:?}", user_id);
            Some(profile.clone())
        } else {
            error!("found user but not profile {:?}", user_id);
            None
        }
    }

}