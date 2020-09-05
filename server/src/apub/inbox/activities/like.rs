use crate::{
<<<<<<< HEAD
  api::{comment::CommentResponse, post::PostResponse},
=======
>>>>>>> 11149ba0
  apub::{
    fetcher::{get_or_fetch_and_insert_comment, get_or_fetch_and_insert_post},
    inbox::shared_inbox::{
      announce_if_community_is_local,
      get_user_from_activity,
      receive_unhandled_activity,
    },
    FromApub,
    PageExt,
  },
  blocking,
<<<<<<< HEAD
  routes::ChatServerParam,
  websocket::{
    server::{SendComment, SendPost},
    UserOperation,
  },
  DbPool,
  LemmyError,
};
use activitystreams::{activity::Like, base::AnyBase, object::Note, prelude::*};
use actix_web::{client::Client, HttpResponse};
=======
  websocket::{
    messages::{SendComment, SendPost},
    UserOperation,
  },
  LemmyContext,
};
use activitystreams::{activity::Like, base::AnyBase, object::Note, prelude::*};
use actix_web::HttpResponse;
use anyhow::Context;
use lemmy_api_structs::{comment::CommentResponse, post::PostResponse};
>>>>>>> 11149ba0
use lemmy_db::{
  comment::{CommentForm, CommentLike, CommentLikeForm},
  comment_view::CommentView,
  post::{PostForm, PostLike, PostLikeForm},
  post_view::PostView,
  Likeable,
};
<<<<<<< HEAD

pub async fn receive_like(
  activity: AnyBase,
  client: &Client,
  pool: &DbPool,
  chat_server: ChatServerParam,
) -> Result<HttpResponse, LemmyError> {
  let like = Like::from_any_base(activity)?.unwrap();
  match like.object().as_single_kind_str() {
    Some("Page") => receive_like_post(like, client, pool, chat_server).await,
    Some("Note") => receive_like_comment(like, client, pool, chat_server).await,
=======
use lemmy_utils::{location_info, LemmyError};

pub async fn receive_like(
  activity: AnyBase,
  context: &LemmyContext,
) -> Result<HttpResponse, LemmyError> {
  let like = Like::from_any_base(activity)?.context(location_info!())?;
  match like.object().as_single_kind_str() {
    Some("Page") => receive_like_post(like, context).await,
    Some("Note") => receive_like_comment(like, context).await,
>>>>>>> 11149ba0
    _ => receive_unhandled_activity(like),
  }
}

<<<<<<< HEAD
async fn receive_like_post(
  like: Like,
  client: &Client,
  pool: &DbPool,
  chat_server: ChatServerParam,
) -> Result<HttpResponse, LemmyError> {
  let user = get_user_from_activity(&like, client, pool).await?;
  let page = PageExt::from_any_base(like.object().to_owned().one().unwrap())?.unwrap();

  let post = PostForm::from_apub(&page, client, pool).await?;

  let post_id = get_or_fetch_and_insert_post(&post.get_ap_id()?, client, pool)
=======
async fn receive_like_post(like: Like, context: &LemmyContext) -> Result<HttpResponse, LemmyError> {
  let user = get_user_from_activity(&like, context).await?;
  let page = PageExt::from_any_base(like.object().to_owned().one().context(location_info!())?)?
    .context(location_info!())?;

  let post = PostForm::from_apub(&page, context, None).await?;

  let post_id = get_or_fetch_and_insert_post(&post.get_ap_id()?, context)
>>>>>>> 11149ba0
    .await?
    .id;

  let like_form = PostLikeForm {
    post_id,
    user_id: user.id,
    score: 1,
  };
<<<<<<< HEAD
  blocking(pool, move |conn| {
    PostLike::remove(conn, &like_form)?;
=======
  let user_id = user.id;
  blocking(context.pool(), move |conn| {
    PostLike::remove(conn, user_id, post_id)?;
>>>>>>> 11149ba0
    PostLike::like(conn, &like_form)
  })
  .await??;

  // Refetch the view
<<<<<<< HEAD
  let post_view = blocking(pool, move |conn| PostView::read(conn, post_id, None)).await??;

  let res = PostResponse { post: post_view };

  chat_server.do_send(SendPost {
    op: UserOperation::CreatePostLike,
    post: res,
    my_id: None,
  });

  announce_if_community_is_local(like, &user, client, pool).await?;
=======
  let post_view = blocking(context.pool(), move |conn| {
    PostView::read(conn, post_id, None)
  })
  .await??;

  let res = PostResponse { post: post_view };

  context.chat_server().do_send(SendPost {
    op: UserOperation::CreatePostLike,
    post: res,
    websocket_id: None,
  });

  announce_if_community_is_local(like, &user, context).await?;
>>>>>>> 11149ba0
  Ok(HttpResponse::Ok().finish())
}

async fn receive_like_comment(
  like: Like,
<<<<<<< HEAD
  client: &Client,
  pool: &DbPool,
  chat_server: ChatServerParam,
) -> Result<HttpResponse, LemmyError> {
  let note = Note::from_any_base(like.object().to_owned().one().unwrap())?.unwrap();
  let user = get_user_from_activity(&like, client, pool).await?;

  let comment = CommentForm::from_apub(&note, client, pool).await?;

  let comment_id = get_or_fetch_and_insert_comment(&comment.get_ap_id()?, client, pool)
=======
  context: &LemmyContext,
) -> Result<HttpResponse, LemmyError> {
  let note = Note::from_any_base(like.object().to_owned().one().context(location_info!())?)?
    .context(location_info!())?;
  let user = get_user_from_activity(&like, context).await?;

  let comment = CommentForm::from_apub(&note, context, None).await?;

  let comment_id = get_or_fetch_and_insert_comment(&comment.get_ap_id()?, context)
>>>>>>> 11149ba0
    .await?
    .id;

  let like_form = CommentLikeForm {
    comment_id,
    post_id: comment.post_id,
    user_id: user.id,
    score: 1,
  };
<<<<<<< HEAD
  blocking(pool, move |conn| {
    CommentLike::remove(conn, &like_form)?;
=======
  let user_id = user.id;
  blocking(context.pool(), move |conn| {
    CommentLike::remove(conn, user_id, comment_id)?;
>>>>>>> 11149ba0
    CommentLike::like(conn, &like_form)
  })
  .await??;

  // Refetch the view
<<<<<<< HEAD
  let comment_view =
    blocking(pool, move |conn| CommentView::read(conn, comment_id, None)).await??;
=======
  let comment_view = blocking(context.pool(), move |conn| {
    CommentView::read(conn, comment_id, None)
  })
  .await??;
>>>>>>> 11149ba0

  // TODO get those recipient actor ids from somewhere
  let recipient_ids = vec![];
  let res = CommentResponse {
    comment: comment_view,
    recipient_ids,
    form_id: None,
  };

<<<<<<< HEAD
  chat_server.do_send(SendComment {
    op: UserOperation::CreateCommentLike,
    comment: res,
    my_id: None,
  });

  announce_if_community_is_local(like, &user, client, pool).await?;
=======
  context.chat_server().do_send(SendComment {
    op: UserOperation::CreateCommentLike,
    comment: res,
    websocket_id: None,
  });

  announce_if_community_is_local(like, &user, context).await?;
>>>>>>> 11149ba0
  Ok(HttpResponse::Ok().finish())
}
