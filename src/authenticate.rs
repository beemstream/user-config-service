use rocket::{
    http::Status,
    info,
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
    let is_valid = request_token.starts_with(&["Bearer"]);

    info!("is token validated {}", is_valid);

    is_valid
}

#[async_trait]
impl<'r> FromRequest<'r> for AccessToken {
    type Error = AccessTokenError;

    async fn from_request(request: &'r rocket::Request<'_>) -> Outcome<Self, Self::Error> {
        let keys: Vec<&str> = request.headers().get("token").collect();
        match keys.len() {
            1 if is_token_valid(keys[0]) => Outcome::Success(AccessToken(keys[0].to_string())),
            _ => Outcome::Failure((Status::Unauthorized, AccessTokenError::Invalid)),
        }
    }
}
