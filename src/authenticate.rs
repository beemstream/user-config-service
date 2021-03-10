use rocket::{
    http::Status,
    request::{FromRequest, Outcome},
};

use async_trait::async_trait;

#[derive(Debug)]
pub struct AccessToken(pub String);

#[derive(Debug)]
pub enum AccessTokenError {
    Missing,
    Invalid,
}

pub fn is_token_valid(token: &str) -> bool {
    let request_token: Vec<&str> = token.split(" ").collect();
    request_token.starts_with(&["Bearer"])
}

#[async_trait]
impl<'a, 'r> FromRequest<'a, 'r> for AccessToken {
    type Error = AccessTokenError;

    async fn from_request(
        request: &'a rocket::Request<'r>,
    ) -> rocket::request::Outcome<Self, Self::Error> {
        let keys: Vec<&str> = request.headers().get("token").collect();
        match keys.len() {
            0 => Outcome::Failure((Status::Unauthorized, AccessTokenError::Missing)),
            1 if is_token_valid(keys[0]) => Outcome::Success(AccessToken(keys[0].to_string())),
            _ => Outcome::Failure((Status::Unauthorized, AccessTokenError::Invalid)),
        }
    }
}
