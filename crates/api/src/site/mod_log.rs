use crate::Perform;
use actix_web::web::Data;
use lemmy_api_common::{
  site::{GetModlog, GetModlogResponse},
  utils::{blocking, check_private_instance, get_local_user_view_from_jwt_opt, is_admin},
};
use lemmy_db_schema::{source::site::Site,
  ModlogActionType,};
use lemmy_db_views_moderator::structs::{
  AdminPurgeCommentView,
  AdminPurgeCommunityView,
  AdminPurgePersonView,
  AdminPurgePostView,
  ModAddCommunityView,
  ModAddView,
  ModBanFromCommunityView,
  ModBanView,
  ModHideCommunityView,
  ModLockPostView,
  ModRemoveCommentView,
  ModRemoveCommunityView,
  ModRemovePostView,
  ModStickyPostView,
  ModTransferCommunityView,
};
use lemmy_utils::{error::LemmyError, ConnectionId};
use lemmy_websocket::LemmyContext;

#[async_trait::async_trait(?Send)]
impl Perform for GetModlog {
  type Response = GetModlogResponse;

  #[tracing::instrument(skip(context, _websocket_id))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<GetModlogResponse, LemmyError> {
    let data: &GetModlog = self;

    let local_user_view =
      get_local_user_view_from_jwt_opt(data.auth.as_ref(), context.pool(), context.secret())
        .await?;

    check_private_instance(&local_user_view, context.pool()).await?;

    let filter_by_action = data.filter_by_action.unwrap_or(ModlogActionType::All);
    let community_id = data.community_id;

    let mod_person_id = data.mod_person_id;
    let other_person_id = data.other_person_id;
    let site = blocking(context.pool(), Site::read_local_site).await??;
    let hide_modlog_names = site.hide_modlog_mod_names && (local_user_view.is_none() || is_admin(&local_user_view.expect("")).is_err());
    let page = data.page;
    let limit = data.limit;
    let removed_posts = if filter_by_action != ModlogActionType::ModRemovePost && filter_by_action != ModlogActionType::All{
      Vec::<ModRemovePostView>::new()
    } else {
      blocking(context.pool(), move |conn| {
        ModRemovePostView::list(
          conn,
          community_id,
          mod_person_id,
          other_person_id,
          page,
          limit,
          hide_modlog_names,
        )
      })
      .await??
    };

    let locked_posts = if filter_by_action != ModlogActionType::ModLockPost && filter_by_action != ModlogActionType::All{
      Vec::<ModLockPostView>::new()
    } else {
      blocking(context.pool(), move |conn| {
        ModLockPostView::list(
          conn,
          community_id,
          mod_person_id,
          other_person_id,
          page,
          limit,
          hide_modlog_names,
        )
      })
      .await??
    };

    let stickied_posts = if filter_by_action != ModlogActionType::ModStickyPost && filter_by_action != ModlogActionType::All{
      Vec::<ModStickyPostView>::new()
    } else {
      blocking(context.pool(), move |conn| {
        ModStickyPostView::list(
          conn,
          community_id,
          mod_person_id,
          other_person_id,
          page,
          limit,
          hide_modlog_names,
        )
      })
      .await??
    };

    let removed_comments = if filter_by_action != ModlogActionType::ModRemoveComment && filter_by_action != ModlogActionType::All{
      Vec::<ModRemoveCommentView>::new()
    } else {
      blocking(context.pool(), move |conn| {
        ModRemoveCommentView::list(
          conn,
          community_id,
          mod_person_id,
          other_person_id,
          page,
          limit,
          hide_modlog_names,
        )
      })
      .await??
    };

    let banned_from_community = if filter_by_action != ModlogActionType::ModBanFromCommunity && filter_by_action != ModlogActionType::All{
      Vec::<ModBanFromCommunityView>::new()
    } else {
      blocking(context.pool(), move |conn| {
        ModBanFromCommunityView::list(
          conn,
          community_id,
          mod_person_id,
          other_person_id,
          page,
          limit,
          hide_modlog_names,
        )
      })
      .await??
    };

    let added_to_community = if filter_by_action != ModlogActionType::ModAddCommunity && filter_by_action != ModlogActionType::All{
      Vec::<ModAddCommunityView>::new()
    } else {
      blocking(context.pool(), move |conn| {
        ModAddCommunityView::list(
          conn,
          community_id,
          mod_person_id,
          other_person_id,
          page,
          limit,
          hide_modlog_names,
        )
      })
      .await??
    };

    let transferred_to_community =
    if filter_by_action != ModlogActionType::ModTransferCommunity && filter_by_action != ModlogActionType::All{
        Vec::<ModTransferCommunityView>::new()
      } else {
        blocking(context.pool(), move |conn| {
          ModTransferCommunityView::list(
            conn,
            community_id,
            mod_person_id,
            other_person_id,
            page,
            limit,
            hide_modlog_names,
          )
        })
        .await??
      };

    let hidden_communities = if filter_by_action != ModlogActionType::ModHideCommunity && filter_by_action != ModlogActionType::All{
      Vec::<ModHideCommunityView>::new()
    } else {
      blocking(context.pool(), move |conn| {
        ModHideCommunityView::list(
          conn,
          community_id,
          mod_person_id,
          page,
          limit,
          hide_modlog_names,
        )
      })
      .await??
    };

    // These arrays are only for the full modlog, when a community isn't given
    let (
      removed_communities,
      banned,
      added,
      admin_purged_persons,
      admin_purged_communities,
      admin_purged_posts,
      admin_purged_comments,
    ) = if data.community_id.is_none() {
      blocking(context.pool(), move |conn| {
        Ok((
          if filter_by_action != ModlogActionType::ModRemoveCommunity && filter_by_action != ModlogActionType::All{
            Vec::<ModRemoveCommunityView>::new()
          } else {
            ModRemoveCommunityView::list(
              conn,
              mod_person_id,
              page,
              limit,
              hide_modlog_names,
            )?
          },
          if filter_by_action != ModlogActionType::ModBan && filter_by_action != ModlogActionType::All{
            Vec::<ModBanView>::new()
          } else {
            ModBanView::list(
              conn,
              mod_person_id,
              other_person_id,
              page,
              limit,
              hide_modlog_names,
            )?
          },
          if filter_by_action != ModlogActionType::ModAdd && filter_by_action != ModlogActionType::All{
            Vec::<ModAddView>::new()
          } else {
            ModAddView::list(
              conn,
              mod_person_id,
              other_person_id,
              page,
              limit,
              hide_modlog_names,
            )?
          },
          AdminPurgePersonView::list(conn, mod_person_id, page, limit)?,
          AdminPurgeCommunityView::list(conn, mod_person_id, page, limit)?,
          AdminPurgePostView::list(conn, mod_person_id, page, limit)?,
          AdminPurgeCommentView::list(conn, mod_person_id, page, limit)?,
        )) as Result<_, LemmyError>
      })
      .await??
    } else {
      Default::default()
    };

    // Return the jwt
    Ok(GetModlogResponse {
      removed_posts,
      locked_posts,
      stickied_posts,
      removed_comments,
      removed_communities,
      banned_from_community,
      banned,
      added_to_community,
      added,
      transferred_to_community,
      admin_purged_persons,
      admin_purged_communities,
      admin_purged_posts,
      admin_purged_comments,
      hidden_communities,
    })
  }
}
