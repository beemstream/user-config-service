#[macro_use]
extern crate diesel;
use authenticate::AccessToken;
use database::favourite_streams::{FavouriteStreamsModel, SavedFavouriteStreamsModel, StreamSource, find_all_favourited_streamers, find_favourite_streamer, insert_favourite_streamer};
use rocket::{State, get, http::Status, launch, post, routes};
use rocket_contrib::databases::diesel::PgConnection;
use rocket_contrib::{database, json::Json};
use serde::{Deserialize, Serialize};
use service::get_profile;

pub mod authenticate;
pub mod schema;
pub mod database;
pub mod service;

#[derive(Debug, Deserialize, Serialize)]
pub struct FavouriteStreamsRequest {
    identifier: String,
    source: StreamSource,
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

#[post("/favourite-streams", data = "<favourite_streams_request>")]
async fn post_favourite_stream(
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

#[get("/favourite-streams")]
async fn get_favourite_streams(
    db_conn: DbConn,
    global_config: State<'_, GlobalConfig>,
    access_token: AccessToken,
) -> Result<Json<Vec<FavouriteStreamResponse>>, Status> {
    let profile = get_profile(&access_token.0, &global_config.auth_url).await?;

    let all_favourited_streams = find_all_favourited_streamers(&db_conn, profile.id).await?;

    let response = all_favourited_streams
        .into_iter()
        .map(|s| FavouriteStreamResponse::from(s))
        .collect();

    Ok(Json(response))
}

#[database("pg_conn")]
pub struct DbConn(PgConnection);

#[derive(Deserialize)]
struct GlobalConfig {
    auth_url: String,
}

#[launch]
fn rocket() -> rocket::Rocket {
    openssl_probe::init_ssl_cert_env_vars();
    env_logger::init();
    let rocket = rocket::ignite();
    let global_config: GlobalConfig = rocket.figment()
        .extract()
        .expect("global config");

    rocket
        .attach(DbConn::fairing())
        .manage(global_config)
        .mount(
            "/stream-config",
            routes![post_favourite_stream, get_favourite_streams],
        )
}
