use crate::PerformCrud;
use actix_web::web::Data;
use lemmy_api_common::{
  community::{CommunityResponse, CreateCommunity},
  get_local_user_view_from_jwt,
  is_admin,
};
use lemmy_apub::{
  generate_apub_endpoint,
  generate_followers_url,
  generate_inbox_url,
  generate_shared_inbox_url,
  EndpointType,
};
use lemmy_db_queries::{diesel_option_overwrite_to_url, ApubObject, Crud, Followable, Joinable};
use lemmy_db_schema::source::{
  community::{
    Community,
    CommunityFollower,
    CommunityFollowerForm,
    CommunityForm,
    CommunityModerator,
    CommunityModeratorForm,
  },
  site::Site,
};
use lemmy_db_views_actor::community_view::CommunityView;
use lemmy_utils::{
  apub::generate_actor_keypair,
  utils::{check_slurs, check_slurs_opt, is_valid_actor_name},
  ApiError,
  ConnectionId,
  LemmyError,
};
use lemmy_websocket::LemmyContext;

#[async_trait::async_trait(?Send)]
impl PerformCrud for CreateCommunity {
  type Response = CommunityResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<CommunityResponse, LemmyError> {
    let data: &CreateCommunity = self;
    let local_user_view = get_local_user_view_from_jwt(&data.auth, context.pool()).await?;

    let site = Site::read(&&context.pool.get().await?, 0)?;
    if site.community_creation_admin_only && is_admin(&local_user_view).is_err() {
      return Err(ApiError::err("only_admins_can_create_communities").into());
    }

    check_slurs(&data.name)?;
    check_slurs(&data.title)?;
    check_slurs_opt(&data.description)?;

    if !is_valid_actor_name(&data.name) {
      return Err(ApiError::err("invalid_community_name").into());
    }

    // Double check for duplicate community actor_ids
    let community_actor_id = generate_apub_endpoint(EndpointType::Community, &data.name)?;
    let actor_id_cloned = community_actor_id.to_owned();
    let community_dupe =
      Community::read_from_apub_id(&&context.pool.get().await?, &actor_id_cloned);
    if community_dupe.is_ok() {
      return Err(ApiError::err("community_already_exists").into());
    }

    // Check to make sure the icon and banners are urls
    let icon = diesel_option_overwrite_to_url(&data.icon)?;
    let banner = diesel_option_overwrite_to_url(&data.banner)?;

    // When you create a community, make sure the user becomes a moderator and a follower
    let keypair = generate_actor_keypair()?;

    let community_form = CommunityForm {
      name: data.name.to_owned(),
      title: data.title.to_owned(),
      description: data.description.to_owned(),
      icon,
      banner,
      nsfw: data.nsfw,
      actor_id: Some(community_actor_id.to_owned()),
      private_key: Some(keypair.private_key),
      public_key: Some(keypair.public_key),
      followers_url: Some(generate_followers_url(&community_actor_id)?),
      inbox_url: Some(generate_inbox_url(&community_actor_id)?),
      shared_inbox_url: Some(Some(generate_shared_inbox_url(&community_actor_id)?)),
      ..CommunityForm::default()
    };

    let inserted_community = Community::create(&&context.pool.get().await?, &community_form)
      .map_err(|_| ApiError::err("community_already_exists"))?;

    // The community creator becomes a moderator
    let community_moderator_form = CommunityModeratorForm {
      community_id: inserted_community.id,
      person_id: local_user_view.person.id,
    };

    let join = CommunityModerator::join(&&context.pool.get().await?, &community_moderator_form);
    if join.is_err() {
      return Err(ApiError::err("community_moderator_already_exists").into());
    }

    // Follow your own community
    let community_follower_form = CommunityFollowerForm {
      community_id: inserted_community.id,
      person_id: local_user_view.person.id,
      pending: false,
    };

    let follow = CommunityFollower::follow(&&context.pool.get().await?, &community_follower_form);
    if follow.is_err() {
      return Err(ApiError::err("community_follower_already_exists").into());
    }

    let person_id = local_user_view.person.id;
    let community_view = CommunityView::read(
      &&context.pool.get().await?,
      inserted_community.id,
      Some(person_id),
    )?;

    Ok(CommunityResponse { community_view })
  }
}
