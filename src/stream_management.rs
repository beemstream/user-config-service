use futures::future::{join, try_join_all};
use rocket::{debug, get, http::Status, post, put, serde::json::Json, State};
use serde::{Deserialize, Serialize};

use crate::{
    authenticate::AccessToken,
    database::stream_management::{
        find_stream_tag, find_stream_title, find_stream_titles, insert_stream_tag,
        insert_stream_title, SavedTagModel, StreamTagModel, StreamTitleModel,
    },
    service::{
        get_channel_information, get_twitch_profile, modify_channel_information,
        replace_stream_tags, ModifyChannelRequest, ReplaceTagsRequest, TwitchUser,
    },
    DbConn, GlobalConfig,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct StreamManagementRequest {
    pub title: StreamTitle,
    pub tags: Vec<StreamTag>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StreamTitle {
    pub title: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StreamTag {
    pub id: String,
    pub name: String,
}

pub fn get_access_token(token: &str) -> String {
    token.replace("Bearer ", "OAuth ")
}

pub async fn get_user(access_token: &AccessToken) -> Result<TwitchUser, Status> {
    let parsed_token = get_access_token(&access_token.0);
    get_twitch_profile(&parsed_token).await
}

#[post("/stream-management", data = "<stream_management_request>")]
pub async fn post_stream_management(
    stream_management_request: Json<StreamManagementRequest>,
    db_conn: DbConn,
    access_token: AccessToken,
) -> Result<Status, Status> {
    let profile: TwitchUser = get_user(&access_token).await?;

    let stream_management_inner = stream_management_request.into_inner();

    let stream_title_model =
        StreamTitleModel::from(&stream_management_inner.title, profile.user_id);

    let stream_title = insert_stream_title(&db_conn, stream_title_model).await?;

    let save_tags = stream_management_inner
        .tags
        .into_iter()
        .map(|t| StreamTagModel::from(&t, stream_title.id))
        .collect::<Vec<StreamTagModel>>();

    debug!("saving tags {:?}", save_tags);

    let futures: Vec<_> = save_tags
        .into_iter()
        .map(|t| insert_stream_tag(&db_conn, t))
        .collect();

    try_join_all(futures).await?;

    // let stream_tag_model = StreamTagModel::from(&stream_management_request.tags, );
    Ok(Status::Ok)
}

#[derive(Debug, Serialize)]
pub struct StreamPreset {
    pub id: i32,
    pub title: String,
    pub tags: Vec<SavedTagModel>,
}

#[get("/stream-management")]
pub async fn get_stream_management(
    db_conn: DbConn,
    access_token: AccessToken,
) -> Result<Json<Vec<StreamPreset>>, Status> {
    debug!("ran through");
    let profile: TwitchUser = get_user(&access_token).await?;
    let titles = find_stream_titles(&db_conn, profile.user_id).await?;

    let mut stream_preset_response = vec![];

    for title in &titles {
        let tags = find_stream_tag(&db_conn, title.clone()).await?;

        let response = StreamPreset {
            id: title.id,
            title: title.title.clone(),
            tags,
        };

        stream_preset_response.push(response);
    }

    Ok(Json(stream_preset_response))
}

#[put("/stream-management/<preset_id>/set")]
pub async fn put_stream_management(
    db_conn: DbConn,
    access_token: AccessToken,
    preset_id: i32,
    global_config: &State<GlobalConfig>,
) -> Result<Status, Status> {
    let profile: TwitchUser = get_user(&access_token).await?;
    let title = find_stream_title(&db_conn, preset_id).await?;
    let tags = find_stream_tag(&db_conn, title.clone()).await?;

    let channel_info = get_channel_information(
        &access_token.0,
        &profile.user_id,
        &global_config.twitch_client_id,
    )
    .await?;

    let channel_modify_request = ModifyChannelRequest {
        game_id: channel_info.game_id.clone(),
        broadcaster_language: channel_info.broadcaster_language.clone(),
        title: title.title,
    };

    let tag_ids: Vec<String> = tags.into_iter().map(|t| t.source_id).collect();

    let replace_tags_request = ReplaceTagsRequest { tag_ids: vec![] };

    let channel_modify = modify_channel_information(
        &access_token.0,
        &profile.user_id,
        &global_config.twitch_client_id,
        channel_modify_request,
    );
    let replace_tags = replace_stream_tags(
        &access_token.0,
        &profile.user_id,
        &global_config.twitch_client_id,
        replace_tags_request,
    );

    let (_, _) = join(channel_modify, replace_tags).await;

    Ok(Status::NoContent)
}
