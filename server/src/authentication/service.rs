use super::command::*;
use super::*;
use actix::{prelude::*, WeakAddr};

pub struct AuthenticationService {
    session_key: Vec<u8>,
}

impl AuthenticationService {
    fn new() -> Self {
        let config = Config::from_env().unwrap();
        AuthenticationService {
            session_key: config.auth.key.as_bytes().into(),
        }
    }

    fn validate_login(&self, login: Login) -> Result<LoginResponse, LoginErrResponse> {
        use jsonwebtoken::{EncodingKey, Header};

        if login.password == "password" {
            let expiry_date = Utc::now() + Duration::hours(1);

            let token = jsonwebtoken::encode(
                &Header::default(),
                &Claims {
                    sub: login.username,
                    exp: expiry_date.timestamp() as usize,
                },
                &EncodingKey::from_secret(&self.session_key),
            )
            .unwrap();

            Ok(LoginResponse {
                message: String::from("welcome in"),
                token,
                status: true,
            })
        } else {
            Err(LoginErrResponse {
                message: String::from("nope, sorry, bad credentials"),
                status: true,
            })
        }
    }

    fn validate_token(&self, token: &str) -> Result<String, String> {
        use jsonwebtoken::{DecodingKey, Validation};

        let config = Config::from_env().unwrap();
        let decoded = jsonwebtoken::decode::<Claims>(
            token,
            &DecodingKey::from_secret(&self.session_key),
            &Validation::default(),
        );

        match decoded {
            Ok(decoded) => Ok(decoded.claims.sub),
            Err(e) => Err(e.to_string()),
        }
    }
}

impl Default for AuthenticationService {
    fn default() -> AuthenticationService {
        AuthenticationService::new()
    }
}

impl Actor for AuthenticationService {
    type Context = Context<Self>;
}

impl Handler<ValidateLogin> for AuthenticationService {
    type Result = MessageResult<ValidateLogin>;

    fn handle(&mut self, ValidateLogin { login }: ValidateLogin, _ctx: &mut Self::Context) -> Self::Result {
        // let CloseRoom(room_id) = request;
        log::trace!("received {:?}", login);
        let valid = self.validate_login(login).is_ok();

        MessageResult(valid)
    }
}

impl SystemService for AuthenticationService {}
impl Supervised for AuthenticationService {}
