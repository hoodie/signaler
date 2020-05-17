use super::*;

#[derive(Debug)]
pub struct AuthResponse<T, P> {
    pub token: T,
    pub profile: P,
}

pub type SimpleAuthResponse = AuthResponse<AuthToken, UserProfile>;
