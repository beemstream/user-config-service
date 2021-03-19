use rocket::{http::Status, post};
use rocket_contrib::json::Json;
use serde::{Deserialize, Serialize};

use crate::{authenticate::AccessToken, database::favourite_streams::StreamSource};

#[derive(Debug, Serialize, Deserialize)]
pub struct StreamManagementRequest {
    title: StreamTitle,
    tags: StreamTags,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StreamTitle {
    title: String,
    source: StreamSource,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StreamTags {
    id: String,
    name: String,
}

#[post("/stream-management", data = "<stream_management_request>")]
pub async fn post_stream_management(
    stream_management_request: Json<StreamManagementRequest>,
    access_token: AccessToken,
) -> Result<Status, Status> {
    Ok(Status::Ok)
}
