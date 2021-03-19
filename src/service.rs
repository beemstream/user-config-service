use isahc::{http::StatusCode, AsyncReadResponseExt};
use log::info;
use rocket::http::Status;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Profile {
    pub id: i32,
}

pub async fn get_profile(access_token: &str, url: &str) -> Result<Profile, Status> {
    let request = isahc::Request::builder()
        .uri(url)
        .method("GET")
        .header("token", access_token)
        .body(())
        .unwrap();

    let mut response = isahc::send_async(request).await.unwrap();

    if response.status() != StatusCode::OK {
        info!("failed to authenticate profile {}", response.status());
        return Err(Status::Unauthorized);
    }

    let json = response.json().await.unwrap();
    Ok(json)
}
