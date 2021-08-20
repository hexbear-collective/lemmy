use crate::Perform;
use actix_web::web::Data;
use lemmy_api_common::{
  check_community_ban,
  check_downvotes_enabled,
  check_person_block,
  get_local_user_view_from_jwt,
  is_mod_or_admin,
  mark_post_as_read,
  post::*,
};
use lemmy_apub::{
  activities::{
    post::create_or_update::CreateOrUpdatePost,
    voting::{
      undo_vote::UndoVote,
      vote::{Vote, VoteType},
    },
    CreateOrUpdateType,
  },
  PostOrComment,
};
use lemmy_db_queries::{source::post::Post_, Crud, Likeable, Saveable};
use lemmy_db_schema::source::{moderator::*, post::*};
use lemmy_db_views::post_view::PostView;
use lemmy_utils::{request::fetch_site_metadata, ApiError, ConnectionId, LemmyError};
use lemmy_websocket::{send::send_post_ws_message, LemmyContext, UserOperation};
use std::convert::TryInto;

#[async_trait::async_trait(?Send)]
impl Perform for CreatePostLike {
  type Response = PostResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    websocket_id: Option<ConnectionId>,
  ) -> Result<PostResponse, LemmyError> {
    let data: &CreatePostLike = self;
    let local_user_view = get_local_user_view_from_jwt(&data.auth, context.pool()).await?;

    // Don't do a downvote if site has downvotes disabled
    check_downvotes_enabled(data.score, context.pool()).await?;

    // Check for a community ban
    let post_id = data.post_id;
    let post = Post::read(&&context.pool.get().await?, post_id)?;

    check_community_ban(local_user_view.person.id, post.community_id, context.pool()).await?;

    check_person_block(local_user_view.person.id, post.creator_id, context.pool()).await?;

    let like_form = PostLikeForm {
      post_id: data.post_id,
      person_id: local_user_view.person.id,
      score: data.score,
    };

    // Remove any likes first
    let person_id = local_user_view.person.id;
    PostLike::remove(&&context.pool.get().await?, person_id, post_id)?;

    let community_id = post.community_id;
    let object = PostOrComment::Post(Box::new(post));

    // Only add the like if the score isnt 0
    let do_add = like_form.score != 0 && (like_form.score == 1 || like_form.score == -1);
    if do_add {
      let like_form2 = like_form.clone();
      let like = PostLike::like(&&context.pool.get().await?, &like_form2);
      if like.is_err() {
        return Err(ApiError::err("couldnt_like_post").into());
      }

      Vote::send(
        &object,
        &local_user_view.person,
        community_id,
        like_form.score.try_into()?,
        context,
      )
      .await?;
    } else {
      // API doesn't distinguish between Undo/Like and Undo/Dislike
      UndoVote::send(
        &object,
        &local_user_view.person,
        community_id,
        VoteType::Like,
        context,
      )
      .await?;
    }

    // Mark the post as read
    mark_post_as_read(person_id, post_id, context.pool()).await?;

    send_post_ws_message(
      data.post_id,
      UserOperation::CreatePostLike,
      websocket_id,
      Some(local_user_view.person.id),
      context,
    )
    .await
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for LockPost {
  type Response = PostResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    websocket_id: Option<ConnectionId>,
  ) -> Result<PostResponse, LemmyError> {
    let data: &LockPost = self;
    let local_user_view = get_local_user_view_from_jwt(&data.auth, context.pool()).await?;

    let post_id = data.post_id;
    let orig_post = Post::read(&&context.pool.get().await?, post_id)?;

    check_community_ban(
      local_user_view.person.id,
      orig_post.community_id,
      context.pool(),
    )
    .await?;

    // Verify that only the mods can lock
    is_mod_or_admin(
      context.pool(),
      local_user_view.person.id,
      orig_post.community_id,
    )
    .await?;

    // Update the post
    let post_id = data.post_id;
    let locked = data.locked;
    let updated_post = Post::update_locked(&&context.pool.get().await?, post_id, locked)?;

    // Mod tables
    let form = ModLockPostForm {
      mod_person_id: local_user_view.person.id,
      post_id: data.post_id,
      locked: Some(locked),
    };
    ModLockPost::create(&&context.pool.get().await?, &form)?;

    // apub updates
    CreateOrUpdatePost::send(
      &updated_post,
      &local_user_view.person,
      CreateOrUpdateType::Update,
      context,
    )
    .await?;

    send_post_ws_message(
      data.post_id,
      UserOperation::LockPost,
      websocket_id,
      Some(local_user_view.person.id),
      context,
    )
    .await
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for StickyPost {
  type Response = PostResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    websocket_id: Option<ConnectionId>,
  ) -> Result<PostResponse, LemmyError> {
    let data: &StickyPost = self;
    let local_user_view = get_local_user_view_from_jwt(&data.auth, context.pool()).await?;

    let post_id = data.post_id;
    let orig_post = Post::read(&&context.pool.get().await?, post_id)?;

    check_community_ban(
      local_user_view.person.id,
      orig_post.community_id,
      context.pool(),
    )
    .await?;

    // Verify that only the mods can sticky
    is_mod_or_admin(
      context.pool(),
      local_user_view.person.id,
      orig_post.community_id,
    )
    .await?;

    // Update the post
    let post_id = data.post_id;
    let stickied = data.stickied;
    let updated_post = Post::update_stickied(&&context.pool.get().await?, post_id, stickied)?;

    // Mod tables
    let form = ModStickyPostForm {
      mod_person_id: local_user_view.person.id,
      post_id: data.post_id,
      stickied: Some(stickied),
    };
    ModStickyPost::create(&&context.pool.get().await?, &form)?;

    // Apub updates
    // TODO stickied should pry work like locked for ease of use
    CreateOrUpdatePost::send(
      &updated_post,
      &local_user_view.person,
      CreateOrUpdateType::Update,
      context,
    )
    .await?;

    send_post_ws_message(
      data.post_id,
      UserOperation::StickyPost,
      websocket_id,
      Some(local_user_view.person.id),
      context,
    )
    .await
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for SavePost {
  type Response = PostResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<PostResponse, LemmyError> {
    let data: &SavePost = self;
    let local_user_view = get_local_user_view_from_jwt(&data.auth, context.pool()).await?;

    let post_saved_form = PostSavedForm {
      post_id: data.post_id,
      person_id: local_user_view.person.id,
    };

    if data.save {
      let save = PostSaved::save(&&context.pool.get().await?, &post_saved_form);
      if save.is_err() {
        return Err(ApiError::err("couldnt_save_post").into());
      }
    } else {
      let unsave = PostSaved::unsave(&&context.pool.get().await?, &post_saved_form);
      if unsave.is_err() {
        return Err(ApiError::err("couldnt_save_post").into());
      }
    }

    let post_id = data.post_id;
    let person_id = local_user_view.person.id;
    let post_view = PostView::read(&&context.pool.get().await?, post_id, Some(person_id))?;

    // Mark the post as read
    mark_post_as_read(person_id, post_id, context.pool()).await?;

    Ok(PostResponse { post_view })
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for GetSiteMetadata {
  type Response = GetSiteMetadataResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<GetSiteMetadataResponse, LemmyError> {
    let data: &Self = self;

    let metadata = fetch_site_metadata(context.client(), &data.url).await?;

    Ok(GetSiteMetadataResponse { metadata })
  }
}
