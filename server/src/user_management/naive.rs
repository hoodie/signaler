use super::*;
use super::user::UserProfile;

use actix::prelude::*;

impl Actor for UserManagement<UserProfile, UsernamePassword> {
    type Context = Context<Self>;
}

type NaiveUserManagment = UserManagement<UserProfile, UsernamePassword>;
impl SystemService for NaiveUserManagment {}
impl Supervised for NaiveUserManagment {}

#[derive(Message)]
#[rtype(result = "Option<UserProfile>")]
pub struct AuthenticationRequest {
    pub credentials: UsernamePassword,
    // pub session_id: SessionId,
}

impl Handler<AuthenticationRequest> for NaiveUserManagment {
    type Result = MessageResult<AuthenticationRequest>;

    fn handle(&mut self, request: AuthenticationRequest, _ctx: &mut Self::Context) -> Self::Result {
        info!("received AuthenticationRequest");

        let AuthenticationRequest {credentials} = request;

        MessageResult(self.associate_user(&credentials))
    }
}


impl Default for NaiveUserManagment {
    fn default() -> Self {
        NaiveUserManager::new()
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct UserDatabase {
    credentials: HashMap<UserId, String>,
    profiles: HashMap<UserId, UserProfile>,
}

#[derive(Debug, Default)]
pub struct NaiveUserManager {
    user_database: UserDatabase
}

impl NaiveUserManager {
    pub fn new() -> NaiveUserManagment {
        let manager = NaiveUserManager {
            user_database: serde_json::from_str(include_str!("../../test_users.json")).unwrap()
        };
        debug!("new NaiveUserManager {:?}", manager);
        UserManagement::new(Box::new(manager))
    }
}

impl UserManaging for NaiveUserManager {
    type UserProfile = UserProfile;
    type Credentials = UsernamePassword;

    fn associate_user(&mut self, credentials: &Self::Credentials) -> Option<UserProfile> {
        let UsernamePassword {username, password} = credentials;

        if Some(password) == self.user_database.credentials.get(username) {
            info!("trace {:?}", credentials);
            if let Some(profile) = self.user_database.profiles.get(username) {
                info!("found profile {:?}", username);
                return Some(profile.clone());
            } else {
                error!("found user but not profile {:?}", username);
            }
        } else {
            info!("not found {:?}", credentials);
        }
        None
    }

}