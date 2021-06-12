use rocket::http::Status;
use rocket_sync_db_pools::diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{favourite_streams::FavouriteStreamsRequest, schema::favourite_streams, DbConn};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum StreamSource {
    Twitch,
    Youtube,
}

impl StreamSource {
    pub fn from(source: StreamSource) -> String {
        match source {
            StreamSource::Twitch => "Twitch",
            StreamSource::Youtube => "Toutube",
        }
        .to_owned()
    }
}

#[derive(Debug, Insertable)]
#[table_name = "favourite_streams"]
pub struct FavouriteStreamsModel {
    pub associated_user: i32,
    pub identifier: String,
    pub source: String,
}

#[derive(Debug, Insertable, Queryable, Serialize)]
#[table_name = "favourite_streams"]
pub struct SavedFavouriteStreamsModel {
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

pub async fn insert_favourite_streamer(
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

pub async fn find_favourite_streamer(
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

pub async fn find_all_favourited_streamers(
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
