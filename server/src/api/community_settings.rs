use super::*;
use crate::{
  api::{APIError, Oper, Perform},
  blocking,
  db::{
    community_settings::{CommunitySettings, CommunitySettingsForm},
    Crud,
  },
  naive_now,
  websocket::{server::SendCommunityRoomMessage, UserOperation, WebsocketInfo},
  DbPool, LemmyError,
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
  pub comment_images: i32,
  pub published: chrono::NaiveDateTime,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EditCommunitySettings {
  pub community_id: i32,
  pub read_only: bool,
  pub private: bool,
  pub post_links: bool,
  pub comment_images: i32,
  pub published: chrono::NaiveDateTime,
  auth: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct EditCommunitySettingsResponse {
  pub read_only: bool,
  pub private: bool,
  pub post_links: bool,
  pub comment_images: i32,
  pub published: chrono::NaiveDateTime,
}

#[async_trait::async_trait(?Send)]
impl Perform for Oper<GetCommunitySettings> {
  type Response = GetCommunitySettingsResponse;

  async fn perform(
    &self,
    pool: &DbPool,
    _websocket_info: Option<WebsocketInfo>,
  ) -> Result<GetCommunitySettingsResponse, LemmyError> {
    let data: &GetCommunitySettings = &self.data;

    /*
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
    */

    let community_id = data.community_id;
    let community_settings = blocking(pool, move |conn| {
      CommunitySettings::read_from_community_id(conn, community_id)
    })
    .await??;

    let res = GetCommunitySettingsResponse {
      read_only: community_settings.read_only,
      private: community_settings.private,
      post_links: community_settings.post_links,
      comment_images: community_settings.comment_images,
      published: naive_now(),
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
      Ok(claims) => claims.claims.id,
      Err(_e) => return Err(APIError::err("no_community_edit_allowed").into()),
    };

    // Verify it's a mod or admin
    let community_id = data.community_id;
    let _: Result<(), LemmyError> = blocking(pool, move |conn| {
      if !User_::read(&conn, user_id)?.is_mod_or_admin(&conn, community_id)? {
        Ok(())
      } else {
        Err(APIError::err("no_community_edit_allowed").into())
      }
    })
    .await?;
    /*
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
    */

    /*
    let community_id = data.community_id;
    let read_community_settings = blocking(pool, move |conn| {
      CommunitySettings::read_from_community_id(conn, community_id)
    }).await??;
    */

    let community_settings_form = CommunitySettingsForm {
      community_id: data.community_id.to_owned(),
      read_only: data.read_only.to_owned(),
      private: data.private.to_owned(),
      post_links: data.post_links.to_owned(),
      comment_images: data.comment_images.to_owned(),
      published: naive_now(),
    };

    let community_id = data.community_id;
    let updated_community_settings = match blocking(pool, move |conn| {
      CommunitySettings::update(conn, community_id, &community_settings_form)
    })
    .await?
    {
      Ok(settings) => settings,
      Err(_e) => return Err(APIError::err("couldnt_update_settings").into()),
    };

    let res = EditCommunitySettingsResponse {
      read_only: updated_community_settings.read_only,
      private: updated_community_settings.private,
      post_links: updated_community_settings.post_links,
      comment_images: updated_community_settings.comment_images,
      published: updated_community_settings.published,
    };

    if let Some(ws) = websocket_info {
      ws.chatserver.do_send(SendCommunityRoomMessage {
        op: UserOperation::EditCommunitySettings,
        response: res.clone(),
        community_id: data.community_id,
        my_id: ws.id,
      });
    }

    // Return the jwt
    Ok(res)
  }
}
