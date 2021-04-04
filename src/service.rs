use isahc::{http::StatusCode, AsyncReadResponseExt, Request};
use rocket::http::Status;
use rocket::info;
use serde::{Deserialize, Serialize};

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

#[derive(Debug, Deserialize)]
pub struct TwitchUser {
    pub client_id: String,
    pub login: String,
    pub scopes: Vec<String>,
    pub user_id: String,
    pub expires_in: i32,
}

pub async fn get_twitch_profile(access_token: &str) -> Result<TwitchUser, Status> {
    let request = Request::builder()
        .uri("https://id.twitch.tv/oauth2/validate")
        .method("GET")
        .header("Authorization", access_token)
        .body(())
        .unwrap();

    let mut response = isahc::send_async(request).await.unwrap();

    if response.status() != StatusCode::OK {
        info!("user not found failed with {:?}", response.status());
        return Err(Status::NotFound);
    }
    Ok(response.json().await.unwrap())
}

#[derive(Debug, Deserialize)]
pub struct TwitchChannelInformation {
    pub broadcaster_id: String,
    pub broadcaster_name: String,
    pub broadcaster_language: Option<String>,
    pub game_id: Option<String>,
    pub game_name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ChannelInformationResponse {
    data: Vec<TwitchChannelInformation>,
}

pub async fn get_channel_information(
    access_token: &str,
    user_id: &str,
    client_id: &str,
) -> Result<TwitchChannelInformation, Status> {
    let request = Request::builder()
        .uri(format!(
            "https://api.twitch.tv/helix/channels?broadcaster_id={}",
            user_id
        ))
        .method("GET")
        .header("Authorization", access_token)
        .header("Client-Id", client_id)
        .header("Content-Type", "application/json")
        .body(())
        .unwrap();

    let mut response = isahc::send_async(request).await.unwrap();

    if response.status() != StatusCode::OK {
        info!("user not found failed with {:?}", response.status());
        return Err(Status::NotFound);
    }
    let mut data: ChannelInformationResponse = response.json().await.unwrap();
    Ok(data.data.remove(0))
}

#[derive(Debug, Serialize)]
pub struct ModifyChannelRequest {
    pub game_id: Option<String>,
    pub broadcaster_language: Option<String>,
    pub title: String,
}

pub async fn modify_channel_information(
    access_token: &str,
    user_id: &str,
    client_id: &str,
    request: ModifyChannelRequest,
) -> Result<Status, Status> {
    let request = Request::builder()
        .uri(format!(
            "https://api.twitch.tv/helix/channels?broadcaster_id={}",
            user_id
        ))
        .method("PATCH")
        .header("Authorization", access_token)
        .header("Client-Id", client_id)
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&request).unwrap())
        .unwrap();

    let response = isahc::send_async(request).await.unwrap();

    if response.status() != StatusCode::OK {
        info!("user not found failed with {:?}", response.status());
        return Err(Status::NotFound);
    }
    Ok(Status::NoContent)
}

#[derive(Debug, Serialize)]
pub struct ReplaceTagsRequest {
    pub tag_ids: Vec<String>,
}

pub async fn replace_stream_tags_empty(
    access_token: &str,
    user_id: &str,
    client_id: &str,
) -> Result<Status, Status> {
    let request = Request::builder()
        .uri(format!(
            "https://api.twitch.tv/helix/streams/tags?broadcaster_id={}",
            user_id
        ))
        .method("PUT")
        .header("Authorization", access_token)
        .header("Client-Id", client_id)
        .header("Content-Type", "application/json")
        .body(())
        .unwrap();

    let response = isahc::send_async(request).await.unwrap();

    if response.status() != StatusCode::OK {
        info!("user not found failed with {:?}", response.status());
        return Err(Status::NotFound);
    }
    Ok(Status::NoContent)
}

pub async fn replace_stream_tags(
    access_token: &str,
    user_id: &str,
    client_id: &str,
    request: ReplaceTagsRequest,
) -> Result<Status, Status> {
    let request = Request::builder()
        .uri(format!(
            "https://api.twitch.tv/helix/streams/tags?broadcaster_id={}",
            user_id
        ))
        .method("PUT")
        .header("Authorization", access_token)
        .header("Client-Id", client_id)
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&request).unwrap())
        .unwrap();

    let response = isahc::send_async(request).await.unwrap();

    if response.status() != StatusCode::OK {
        info!("user not found failed with {:?}", response.status());
        return Err(Status::NotFound);
    }
    Ok(Status::NoContent)
}
