use actix_web::web::Data;

use lemmy_api_structs::{community_settings::*, APIError};
use lemmy_db::{
  community_settings::{CommunitySettings, CommunitySettingsForm},
  naive_now, Crud,
};
use lemmy_utils::{ConnectionId, LemmyError};

use crate::{
  api::{get_user_from_jwt, is_mod_or_admin, Perform},
  blocking,
  websocket::{messages::SendCommunityRoomMessage, UserOperation},
  LemmyContext,
};

#[async_trait::async_trait(?Send)]
impl Perform for GetCommunitySettings {
  type Response = GetCommunitySettingsResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<GetCommunitySettingsResponse, LemmyError> {
    let data: &GetCommunitySettings = &self;

    let community_id = data.community_id;
    let community_settings = match blocking(context.pool(), move |conn| {
      CommunitySettings::read_from_community_id(conn, community_id)
    })
    .await?
    {
      Ok(community_settings) => community_settings,
      Err(_e) => return Err(APIError::err("couldnt_find_community").into()),
    };

    let res = GetCommunitySettingsResponse {
      read_only: community_settings.read_only,
      private: community_settings.private,
      post_links: community_settings.post_links,
      comment_images: community_settings.comment_images,
      published: naive_now(),
      allow_as_default: community_settings.allow_as_default,
    };

    // Return the jwt
    Ok(res)
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for EditCommunitySettings {
  type Response = EditCommunitySettingsResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    websocket_id: Option<ConnectionId>,
  ) -> Result<EditCommunitySettingsResponse, LemmyError> {
    let data: &EditCommunitySettings = &self;
    let user = get_user_from_jwt(&data.auth, context.pool()).await?;
    is_mod_or_admin(context.pool(), user.id, data.community_id).await?;

    let community_settings_form = CommunitySettingsForm {
      id: data.community_id.to_owned(),
      read_only: data.read_only.to_owned(),
      private: data.private.to_owned(),
      post_links: data.post_links.to_owned(),
      comment_images: data.comment_images.to_owned(),
      allow_as_default: data.allow_as_default.to_owned(),
    };

    let community_id = data.community_id;
    let updated_community_settings = match blocking(context.pool(), move |conn| {
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
      allow_as_default: updated_community_settings.allow_as_default,
    };

    context.chat_server().do_send(SendCommunityRoomMessage {
      op: UserOperation::BanFromCommunity,
      response: res.clone(),
      community_id,
      websocket_id,
    });

    Ok(res)
  }
}
