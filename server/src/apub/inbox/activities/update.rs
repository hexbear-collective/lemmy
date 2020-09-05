use crate::{
<<<<<<< HEAD
  api::{
    comment::{send_local_notifs, CommentResponse},
    post::PostResponse,
  },
=======
  api::comment::send_local_notifs,
>>>>>>> 11149ba0
  apub::{
    fetcher::{get_or_fetch_and_insert_comment, get_or_fetch_and_insert_post},
    inbox::shared_inbox::{
      announce_if_community_is_local,
      get_user_from_activity,
      receive_unhandled_activity,
    },
<<<<<<< HEAD
=======
    ActorType,
>>>>>>> 11149ba0
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
use activitystreams::{activity::Update, base::AnyBase, object::Note, prelude::*};
use actix_web::{client::Client, HttpResponse};
=======
  websocket::{
    messages::{SendComment, SendPost},
    UserOperation,
  },
  LemmyContext,
};
use activitystreams::{activity::Update, base::AnyBase, object::Note, prelude::*};
use actix_web::HttpResponse;
use anyhow::Context;
use lemmy_api_structs::{comment::CommentResponse, post::PostResponse};
>>>>>>> 11149ba0
use lemmy_db::{
  comment::{Comment, CommentForm},
  comment_view::CommentView,
  post::{Post, PostForm},
  post_view::PostView,
  Crud,
};
<<<<<<< HEAD
use lemmy_utils::scrape_text_for_mentions;

pub async fn receive_update(
  activity: AnyBase,
  client: &Client,
  pool: &DbPool,
  chat_server: ChatServerParam,
) -> Result<HttpResponse, LemmyError> {
  let update = Update::from_any_base(activity)?.unwrap();
  match update.object().as_single_kind_str() {
    Some("Page") => receive_update_post(update, client, pool, chat_server).await,
    Some("Note") => receive_update_comment(update, client, pool, chat_server).await,
=======
use lemmy_utils::{location_info, scrape_text_for_mentions, LemmyError};

pub async fn receive_update(
  activity: AnyBase,
  context: &LemmyContext,
) -> Result<HttpResponse, LemmyError> {
  let update = Update::from_any_base(activity)?.context(location_info!())?;

  // ensure that update and actor come from the same instance
  let user = get_user_from_activity(&update, context).await?;
  update.id(user.actor_id()?.domain().context(location_info!())?)?;

  match update.object().as_single_kind_str() {
    Some("Page") => receive_update_post(update, context).await,
    Some("Note") => receive_update_comment(update, context).await,
>>>>>>> 11149ba0
    _ => receive_unhandled_activity(update),
  }
}

async fn receive_update_post(
  update: Update,
<<<<<<< HEAD
  client: &Client,
  pool: &DbPool,
  chat_server: ChatServerParam,
) -> Result<HttpResponse, LemmyError> {
  let user = get_user_from_activity(&update, client, pool).await?;
  let page = PageExt::from_any_base(update.object().to_owned().one().unwrap())?.unwrap();

  let post = PostForm::from_apub(&page, client, pool).await?;

  let post_id = get_or_fetch_and_insert_post(&post.get_ap_id()?, client, pool)
    .await?
    .id;

  blocking(pool, move |conn| Post::update(conn, post_id, &post)).await??;

  // Refetch the view
  let post_view = blocking(pool, move |conn| PostView::read(conn, post_id, None)).await??;

  let res = PostResponse { post: post_view };

  chat_server.do_send(SendPost {
    op: UserOperation::EditPost,
    post: res,
    my_id: None,
  });

  announce_if_community_is_local(update, &user, client, pool).await?;
=======
  context: &LemmyContext,
) -> Result<HttpResponse, LemmyError> {
  let user = get_user_from_activity(&update, context).await?;
  let page = PageExt::from_any_base(update.object().to_owned().one().context(location_info!())?)?
    .context(location_info!())?;

  let post = PostForm::from_apub(&page, context, Some(user.actor_id()?)).await?;

  let original_post_id = get_or_fetch_and_insert_post(&post.get_ap_id()?, context)
    .await?
    .id;

  blocking(context.pool(), move |conn| {
    Post::update(conn, original_post_id, &post)
  })
  .await??;

  // Refetch the view
  let post_view = blocking(context.pool(), move |conn| {
    PostView::read(conn, original_post_id, None)
  })
  .await??;

  let res = PostResponse { post: post_view };

  context.chat_server().do_send(SendPost {
    op: UserOperation::EditPost,
    post: res,
    websocket_id: None,
  });

  announce_if_community_is_local(update, &user, context).await?;
>>>>>>> 11149ba0
  Ok(HttpResponse::Ok().finish())
}

async fn receive_update_comment(
  update: Update,
<<<<<<< HEAD
  client: &Client,
  pool: &DbPool,
  chat_server: ChatServerParam,
) -> Result<HttpResponse, LemmyError> {
  let note = Note::from_any_base(update.object().to_owned().one().unwrap())?.unwrap();
  let user = get_user_from_activity(&update, client, pool).await?;

  let comment = CommentForm::from_apub(&note, client, pool).await?;

  let comment_id = get_or_fetch_and_insert_comment(&comment.get_ap_id()?, client, pool)
    .await?
    .id;

  let updated_comment = blocking(pool, move |conn| {
    Comment::update(conn, comment_id, &comment)
=======
  context: &LemmyContext,
) -> Result<HttpResponse, LemmyError> {
  let note = Note::from_any_base(update.object().to_owned().one().context(location_info!())?)?
    .context(location_info!())?;
  let user = get_user_from_activity(&update, context).await?;

  let comment = CommentForm::from_apub(&note, context, Some(user.actor_id()?)).await?;

  let original_comment_id = get_or_fetch_and_insert_comment(&comment.get_ap_id()?, context)
    .await?
    .id;

  let updated_comment = blocking(context.pool(), move |conn| {
    Comment::update(conn, original_comment_id, &comment)
>>>>>>> 11149ba0
  })
  .await??;

  let post_id = updated_comment.post_id;
<<<<<<< HEAD
  let post = blocking(pool, move |conn| Post::read(conn, post_id)).await??;

  let mentions = scrape_text_for_mentions(&updated_comment.content);
  let recipient_ids =
    send_local_notifs(mentions, updated_comment, &user, post, pool, false).await?;

  // Refetch the view
  let comment_view =
    blocking(pool, move |conn| CommentView::read(conn, comment_id, None)).await??;
=======
  let post = blocking(context.pool(), move |conn| Post::read(conn, post_id)).await??;

  let mentions = scrape_text_for_mentions(&updated_comment.content);
  let recipient_ids = send_local_notifs(
    mentions,
    updated_comment,
    &user,
    post,
    context.pool(),
    false,
  )
  .await?;

  // Refetch the view
  let comment_view = blocking(context.pool(), move |conn| {
    CommentView::read(conn, original_comment_id, None)
  })
  .await??;
>>>>>>> 11149ba0

  let res = CommentResponse {
    comment: comment_view,
    recipient_ids,
    form_id: None,
  };

<<<<<<< HEAD
  chat_server.do_send(SendComment {
    op: UserOperation::EditComment,
    comment: res,
    my_id: None,
  });

  announce_if_community_is_local(update, &user, client, pool).await?;
=======
  context.chat_server().do_send(SendComment {
    op: UserOperation::EditComment,
    comment: res,
    websocket_id: None,
  });

  announce_if_community_is_local(update, &user, context).await?;
>>>>>>> 11149ba0
  Ok(HttpResponse::Ok().finish())
}
