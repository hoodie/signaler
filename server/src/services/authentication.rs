use chrono::{Duration, Utc};

use crate::config::Config;

pub mod routes {
    use super::*;
    use actix_web::http::StatusCode;
    use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder};

    #[post("/login")]
    async fn login(user: web::Json<Login>) -> HttpResponse {
        let validated = validate_login(user.into_inner());
        match validated {
            // TODO: this needs to be an `Into<HttpResponse>`
            Ok(resp) => HttpResponse::Ok().json(resp),
            Err(resp) => HttpResponse::Ok().status(StatusCode::from_u16(401).unwrap()).json(resp),
        }
    }

    #[get("/profile")]
    async fn profile(req: HttpRequest) -> impl Responder {
        let authentication = dbg!(req
            .headers())
            .get("Authorization" )
            .unwrap();
        let token = authentication
            .to_str()
            .unwrap()
            .split("Bearer").nth(1)
            .map(str::trim);

        if let Some(validated) = dbg!(token).map(validate_token) {
            match validated {
                // TODO: this needs to be an `Into<HttpResponse>`
                Ok(resp) => HttpResponse::Ok().json(resp),
                Err(resp) => HttpResponse::Ok().status(StatusCode::from_u16(401).unwrap()).json(resp),
            }
        } else {
            HttpResponse::Ok()
                .status(StatusCode::from_u16(401).unwrap())
                .body(String::from("No Bearer 🐨"))
        }
    }

    pub fn init(cfg: &mut web::ServiceConfig) {
        cfg.service(login);
        cfg.service(profile);
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Login {
    username: String,
    password: String,
}

#[derive(Default, Debug, serde::Deserialize, serde::Serialize)]
pub struct LoginResponse {
    message: String,
    token: String,
    status: bool,
}

#[derive(Default, Debug, serde::Deserialize, serde::Serialize)]
pub struct LoginErrResponse {
    message: String,
    status: bool,
}

#[derive(Default, Debug, serde::Deserialize, serde::Serialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

fn validate_login(login: Login) -> Result<LoginResponse, LoginErrResponse> {
    use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation};

    if login.password == "password" {
        let config = Config::from_env().unwrap();
        let expiry_date = Utc::now() + Duration::hours(1);

        let token = jsonwebtoken::encode(
            &Header::default(),
            &Claims {
                sub: login.username,
                exp: expiry_date.timestamp() as usize,
            },
            &EncodingKey::from_secret(&config.auth.key.as_bytes()),
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

fn validate_token(token: &str) -> Result<String, String> {
    use jsonwebtoken::{DecodingKey, Validation};

    let config = Config::from_env().unwrap();
    let decoded = jsonwebtoken::decode::<Claims>(
        token,
        // &DecodingKey::from_secret(config.auth.key.as_bytes()),
        &DecodingKey::from_secret(b"this is not the correct secret"),
        &Validation::default()
    );

    match decoded {
        Ok(decoded) => Ok(decoded.claims.sub),
        Err(e) => Err(e.to_string()),
    }
}
