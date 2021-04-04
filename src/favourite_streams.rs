use rocket::{debug, get, http::Status, post, State};
use rocket_contrib::json::Json;
use serde::{Deserialize, Serialize};

use crate::{
    authenticate::AccessToken,
    database::favourite_streams::{
        find_all_favourited_streamers, find_favourite_streamer, insert_favourite_streamer,
        FavouriteStreamsModel, SavedFavouriteStreamsModel, StreamSource,
    },
    service::get_profile,
    DbConn, GlobalConfig,
};

#[derive(Debug, Deserialize, Serialize)]
pub struct FavouriteStreamsRequest {
    pub identifier: String,
    pub source: StreamSource,
}

#[derive(Debug, Serialize)]
pub struct FavouriteStreamResponse {
    pub identifier: String,
    pub source: String,
}

impl FavouriteStreamResponse {
    pub fn from(saved_favourited_streamer: SavedFavouriteStreamsModel) -> Self {
        Self {
            identifier: saved_favourited_streamer.identifier,
            source: saved_favourited_streamer.source,
        }
    }
}

#[get("/favourite-streams")]
pub async fn get_favourite_streams(
    db_conn: DbConn,
    global_config: State<'_, GlobalConfig>,
    access_token: AccessToken,
) -> Result<Json<Vec<FavouriteStreamResponse>>, Status> {
    debug!("got token {}", &access_token.0);
    let profile = get_profile(&access_token.0, &global_config.auth_url).await?;

    let all_favourited_streams = find_all_favourited_streamers(&db_conn, profile.id).await?;

    let response = all_favourited_streams
        .into_iter()
        .map(|s| FavouriteStreamResponse::from(s))
        .collect();

    Ok(Json(response))
}

#[post("/favourite-streams", data = "<favourite_streams_request>")]
pub async fn post_favourite_stream(
    db_conn: DbConn,
    favourite_streams_request: Json<FavouriteStreamsRequest>,
    global_config: State<'_, GlobalConfig>,
    access_token: AccessToken,
) -> Result<Status, Status> {
    let profile = get_profile(&access_token.0, &global_config.auth_url).await?;

    let favourite_stream_unpacked = favourite_streams_request.into_inner();
    let has_found_conflict = find_favourite_streamer(
        &db_conn,
        profile.id,
        favourite_stream_unpacked.identifier.clone(),
        StreamSource::from(favourite_stream_unpacked.source.clone()),
    )
    .await?;

    match has_found_conflict > 0 {
        true => Err(Status::Conflict),
        false => {
            insert_favourite_streamer(
                &db_conn,
                FavouriteStreamsModel::from(favourite_stream_unpacked, profile.id),
            )
            .await?;

            Ok(Status::Created)
        }
    }
}
