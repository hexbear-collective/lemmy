use super::*;
use crate::{
  api::{APIError, Oper, Perform},
  blocking,
  db::community_settings::CommunitySettings,
  naive_now,
  websocket::{
    WebsocketInfo,
  },
  DbPool,
  LemmyError,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct GetCommunitySettings {
  pub community_id: i32,
  auth: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct GetCommunitySettingsResponse {
  pub read_only: bool,
  pub private: bool,
  pub post_links: bool,
  pub post_images: bool,
  pub comment_images: i32,
  pub published: Option<chrono::NaiveDateTime>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EditCommunitySettings {
  pub community_id: i32,
  pub read_only: bool,
  pub private: bool,
  pub post_links: bool,
  pub post_images: bool,
  pub comment_images: i32,
  pub published: chrono::NaiveDateTime,
  auth: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct EditCommunitySettingsResponse {
  pub read_only: bool,
  pub private: bool,
  pub post_links: bool,
  pub post_images: bool,
  pub comment_images: i32,
  pub published: chrono::NaiveDateTime,
}

#[async_trait::async_trait(?Send)]
impl Perform for Oper<GetCommunitySettings> {
  type Response = GetCommunitySettingsResponse;

  async fn perform(
    &self,
    pool: &DbPool,
    websocket_info: Option<WebsocketInfo>,
  ) -> Result<GetCommunitySettingsResponse, LemmyError> {
    let data: &GetCommunitySettings = &self.data;

    let user_id: Option<i32> = match &data.auth {
      Some(auth) => match Claims::decode(&auth) {
        Ok(claims) => {
          let user_id = claims.claims.id;
          Some(user_id)
        }
        Err(_e) => None,
      },
      None => None,
    };

    let community_id = data.community_id;
    let community_settings = blocking(pool, move |conn| {
      CommunitySettings::read_from_community_id(conn, community_id)
    }).await??;

    let _community_settings_id = community_settings.id;

    let res = GetCommunitySettingsResponse {
      read_only: community_settings.read_only,
      private: community_settings.private,
      post_links: community_settings.post_links,
      post_images: community_settings.post_images,
      comment_images: community_settings.comment_images,
      published: Some(naive_now()),
    };

    // Return the jwt
    Ok(res)
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for Oper<EditCommunitySettings> {
  type Response = EditCommunitySettingsResponse;

  async fn perform(
    &self,
    pool: &DbPool,
    websocket_info: Option<WebsocketInfo>,
  ) -> Result<EditCommunitySettingsResponse, LemmyError> {
    let data: &EditCommunitySettings = &self.data;

    let user_id: i32 = match Claims::decode(&data.auth) {
      Ok(claims) => {
        claims.claims.id
      }
      Err(_e) => return Err(APIError::err("settings_no_permission").into()),
    };

    // Verify it's a mod or admin
    let community_id = data.community_id;
    let mut editors: Vec<i32> = Vec::new();
    editors.append(
      &mut blocking(pool, move |conn| {
        CommunityModeratorView::for_community(conn, community_id)
          .map(|v| v.into_iter().map(|m| m.user_id).collect())
      })
      .await??,
    );
    editors.append(
      &mut blocking(pool, move |conn| {
        UserView::admins(conn).map(|v| v.into_iter().map(|a| a.id).collect())
      })
      .await??,
    );
    if !editors.contains(&user_id) {
      return Err(APIError::err("no_post_edit_allowed").into());
    }

    let community_id = data.community_id;
    let community_settings = blocking(pool, move |conn| {
      CommunitySettings::read_from_community_id(conn, community_id)
    }).await??;

    let res = EditCommunitySettingsResponse {
      read_only: data.read_only,
      private: data.private,
      post_links: data.post_links,
      post_images: data.post_images,
      comment_images: data.comment_images,
      published: naive_now(),
    };

    // Return the jwt
    Ok(res)
  }
}
