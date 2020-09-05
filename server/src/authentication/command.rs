use actix::prelude::*;

use super::*;

#[derive(Message, Debug)]
#[rtype(result = "bool")]
pub struct ValidateLogin {
    pub login: Login,
}
