#[macro_use]
extern crate diesel;
use authenticate::AccessToken;
use isahc::{http::StatusCode, AsyncReadResponseExt};
use rocket::http::Status;
use rocket::*;
use rocket_contrib::databases::diesel::prelude::*;
use rocket_contrib::databases::diesel::PgConnection;
use rocket_contrib::{database, json::Json};
use schema::favourite_streams;
use serde::{Deserialize, Serialize};

pub mod authenticate;
pub mod schema;

#[derive(Debug, Clone, Deserialize)]
enum StreamSource {
    Twitch,
    Youtube
}

impl StreamSource {
    pub fn from(source: StreamSource) -> String {
        match source {
            StreamSource::Twitch => "Twitch",
            StreamSource::Youtube => "Toutube"
        }.to_owned()
    }
}


#[derive(Debug, Deserialize)]
struct FavouriteStreamsRequest {
    identifier: String,
    source: StreamSource,
}

#[derive(Debug, Insertable)]
#[table_name = "favourite_streams"]
struct FavouriteStreamsModel {
    pub associated_user: i32,
    pub identifier: String,
    pub source: String,
}

#[derive(Debug, Insertable, Queryable, Serialize)]
#[table_name = "favourite_streams"]
struct SavedFavouriteStreamsModel {
    pub id: i32,
    pub associated_user: i32,
    pub identifier: String,
    pub source: String,
}

impl FavouriteStreamsModel {
    pub fn from(streamer: FavouriteStreamsRequest, user: i32) -> Self {
        Self {
            associated_user: user,
            identifier: streamer.identifier,
            source: StreamSource::from(streamer.source),
        }
    }

}

#[derive(Debug, Serialize)]
struct FavouriteStreamResponse {
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
async fn favourite_stream(
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
    debug!("logging in with {}", &access_token.0);

    let profile = get_profile(&access_token.0, &global_config.auth_url).await?;

    let all_favourited_streams = find_all_favourited_streamers(&db_conn, profile.id).await?;

    let response = all_favourited_streams.into_iter().map(|s| FavouriteStreamResponse::from(s)).collect();

    Ok(Json(response))
}

async fn insert_favourite_streamer(
    db_conn: &DbConn,
    streamer: FavouriteStreamsModel,
) -> Result<usize, Status> {
    db_conn
        .run(|c| {
            diesel::insert_into(favourite_streams::table)
                .values(streamer)
                .execute(c)
                .map_err(|_| Status::NotFound)
        })
        .await
}

async fn find_favourite_streamer(
    db_conn: &DbConn,
    associated_user: i32,
    streamer: String,
    source: String,
) -> Result<usize, Status> {
    db_conn
        .run(move |c| {
            favourite_streams::table
                .filter(favourite_streams::associated_user.eq(associated_user))
                .filter(favourite_streams::identifier.eq(streamer))
                .filter(favourite_streams::source.eq(source))
                .execute(c)
                .map_err(|_| Status::InternalServerError)
        })
        .await
}

async fn find_all_favourited_streamers(
    db_conn: &DbConn,
    associated_user: i32,
) -> Result<Vec<SavedFavouriteStreamsModel>, Status> {
    db_conn
        .run(move |c| {
            favourite_streams::table
                .filter(favourite_streams::associated_user.eq(associated_user))
                .get_results::<SavedFavouriteStreamsModel>(c)
                .map_err(|_| Status::InternalServerError)
        })
        .await
}

#[derive(Deserialize)]
pub struct Profile {
    id: i32,
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
        Err(Status::Unauthorized)
    } else {
        let json = response.json().await.unwrap();
        Ok(json)
    }
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
    let figment = rocket.figment();

    let global_config: GlobalConfig = figment.extract().expect("global config");

    rocket
        .attach(DbConn::fairing())
        .manage(global_config)
        .mount(
            "/stream-config",
            routes![favourite_stream, get_favourite_streams],
        )
}
