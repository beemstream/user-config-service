#[macro_use]
extern crate rocket;
#[macro_use]
extern crate diesel;
use rocket::http::Status;
use rocket::*;
use rocket_contrib::databases::diesel::prelude::*;
use rocket_contrib::databases::diesel::PgConnection;
use rocket_contrib::{database, json::Json};
use schema::favourite_streams;
use serde::{Deserialize, Serialize};

pub mod schema;

#[derive(Debug, Deserialize, Serialize)]
struct FavouriteStreamsRequest {
    identifier: String,
    source: String,
}

#[derive(Debug, Insertable)]
#[table_name = "favourite_streams"]
struct FavouriteStreamsModel {
    pub associated_user: String,
    pub identifier: String,
    pub source: String,
}

impl FavouriteStreamsModel {
    pub fn from(streamer: FavouriteStreamsRequest) -> Self {
        Self {
            associated_user: "beemstreamofficial".to_owned(),
            identifier: streamer.identifier,
            source: streamer.source,
        }
    }
}

#[post("/favourite-streams", data = "<favourite_streams_request>")]
async fn favourite_stream(
    db_conn: DbConn,
    favourite_streams_request: Json<FavouriteStreamsRequest>,
) -> Result<Status, Status> {
    let favourite_stream_unpacked = favourite_streams_request.into_inner();
    let has_found_conflict = find_favourite_streamer(
        &db_conn,
        "beemstreamofficial".to_owned(),
        favourite_stream_unpacked.identifier.clone(),
        favourite_stream_unpacked.source.clone(),
    )
    .await
    .unwrap();

    match has_found_conflict > 0 {
        true => Err(Status::Conflict),
        false => {
            insert_favourite_streamer(
                &db_conn,
                FavouriteStreamsModel::from(favourite_stream_unpacked),
            )
            .await
            .unwrap();

            Ok(Status::Created)
        }
    }
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
    associated_user: String,
    streamer: String,
    source: String,
) -> Result<usize, Status> {
    db_conn
        .run(|c| {
            favourite_streams::table
                .filter(favourite_streams::associated_user.eq(associated_user))
                .filter(favourite_streams::identifier.eq(streamer))
                .filter(favourite_streams::source.eq(source))
                .execute(c)
                .map_err(|_| Status::InternalServerError)
        })
        .await
}

#[database("pg_conn")]
pub struct DbConn(PgConnection);

#[launch]
fn rocket() -> rocket::Rocket {
    rocket::ignite()
        .attach(DbConn::fairing())
        .mount("/stream-config", routes![favourite_stream])
}
