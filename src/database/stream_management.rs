use crate::{
    schema::{stream_tag, stream_title},
    stream_management::{StreamTag, StreamTitle},
    DbConn,
};
use rocket::http::Status;
use rocket_contrib::databases::diesel::prelude::*;
use serde::Serialize;

#[derive(Debug, Insertable, Queryable)]
#[table_name = "stream_title"]
pub struct StreamTitleModel {
    pub associated_user: String,
    pub title: String,
}

impl StreamTitleModel {
    pub fn from(stream_title_request: &StreamTitle, user_id: String) -> Self {
        Self {
            associated_user: user_id,
            title: stream_title_request.title.clone(),
        }
    }
}

#[derive(Debug, Insertable, Queryable)]
#[table_name = "stream_tag"]
pub struct StreamTagModel {
    pub associated_title: i32,
    pub source_id: String,
    pub name: String,
}

impl StreamTagModel {
    pub fn from(stream_tag_request: &StreamTag, title_id: i32) -> Self {
        Self {
            associated_title: title_id,
            source_id: stream_tag_request.id.clone(),
            name: stream_tag_request.name.clone(),
        }
    }
}

pub async fn insert_stream_title(
    db_conn: &DbConn,
    stream_title: StreamTitleModel,
) -> Result<SavedTitleModel, Status> {
    db_conn
        .run(|c| {
            diesel::insert_into(stream_title::table)
                .values(stream_title)
                .get_result::<SavedTitleModel>(c)
                .map_err(|_| Status::NotFound)
        })
        .await
}

pub async fn insert_stream_tag(
    db_conn: &DbConn,
    stream_tag: StreamTagModel,
) -> Result<usize, Status> {
    db_conn
        .run(|c| {
            diesel::insert_into(stream_tag::table)
                .values(stream_tag)
                .execute(c)
                .map_err(|_| Status::NotFound)
        })
        .await
}

pub struct StreamPreset {
    title: String,
    tags: Vec<StreamTagModel>,
}

#[derive(Debug, Insertable, Queryable)]
#[table_name = "stream_title"]
pub struct SavedTitleModel {
    pub id: i32,
    pub associated_user: String,
    pub title: String,
}

pub async fn find_stream_titles(
    db_conn: &DbConn,
    id: String,
) -> Result<Vec<SavedTitleModel>, Status> {
    db_conn
        .run(|c| {
            stream_title::table
                .filter(stream_title::associated_user.eq(id))
                .get_results::<SavedTitleModel>(c)
                .map_err(|_| Status::NotFound)
        })
        .await
}

pub async fn find_stream_title(db_conn: &DbConn, id: i32) -> Result<SavedTitleModel, Status> {
    db_conn
        .run(move |c| {
            stream_title::table
                .filter(stream_title::id.eq(id))
                .get_result::<SavedTitleModel>(c)
                .map_err(|_| Status::NotFound)
        })
        .await
}

#[derive(Debug, Insertable, Queryable, Serialize)]
#[table_name = "stream_tag"]
pub struct SavedTagModel {
    pub id: i32,
    pub associated_title: i32,
    pub source_id: String,
    pub name: String,
}

pub async fn find_stream_tag(
    db_conn: &DbConn,
    title_id: i32,
) -> Result<Vec<SavedTagModel>, Status> {
    db_conn
        .run(move |c| {
            stream_tag::table
                .filter(stream_tag::associated_title.eq(title_id))
                .get_results::<SavedTagModel>(c)
                .map_err(|_| Status::NotFound)
        })
        .await
}
