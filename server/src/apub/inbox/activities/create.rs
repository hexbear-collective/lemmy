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
use activitystreams::{activity::Create, base::AnyBase, object::Note, prelude::*};
use actix_web::{client::Client, HttpResponse};
=======
  websocket::{
    messages::{SendComment, SendPost},
    UserOperation,
  },
  LemmyContext,
};
use activitystreams::{activity::Create, base::AnyBase, object::Note, prelude::*};
use actix_web::HttpResponse;
use anyhow::Context;
use lemmy_api_structs::{comment::CommentResponse, post::PostResponse};
>>>>>>> 11149ba0
use lemmy_db::{
  comment::{Comment, CommentForm},
  comment_view::CommentView,
  post::{Post, PostForm},
  post_view::PostView,
<<<<<<< HEAD
  Crud,
};
use lemmy_utils::scrape_text_for_mentions;

pub async fn receive_create(
  activity: AnyBase,
  client: &Client,
  pool: &DbPool,
  chat_server: ChatServerParam,
) -> Result<HttpResponse, LemmyError> {
  let create = Create::from_any_base(activity)?.unwrap();
  match create.object().as_single_kind_str() {
    Some("Page") => receive_create_post(create, client, pool, chat_server).await,
    Some("Note") => receive_create_comment(create, client, pool, chat_server).await,
=======
};
use lemmy_utils::{location_info, scrape_text_for_mentions, LemmyError};

pub async fn receive_create(
  activity: AnyBase,
  context: &LemmyContext,
) -> Result<HttpResponse, LemmyError> {
  let create = Create::from_any_base(activity)?.context(location_info!())?;

  // ensure that create and actor come from the same instance
  let user = get_user_from_activity(&create, context).await?;
  create.id(user.actor_id()?.domain().context(location_info!())?)?;

  match create.object().as_single_kind_str() {
    Some("Page") => receive_create_post(create, context).await,
    Some("Note") => receive_create_comment(create, context).await,
>>>>>>> 11149ba0
    _ => receive_unhandled_activity(create),
  }
}

async fn receive_create_post(
  create: Create,
<<<<<<< HEAD
  client: &Client,
  pool: &DbPool,
  chat_server: ChatServerParam,
) -> Result<HttpResponse, LemmyError> {
  let user = get_user_from_activity(&create, client, pool).await?;
  let page = PageExt::from_any_base(create.object().to_owned().one().unwrap())?.unwrap();

  let post = PostForm::from_apub(&page, client, pool).await?;

  let inserted_post = blocking(pool, move |conn| Post::create(conn, &post)).await??;

  // Refetch the view
  let inserted_post_id = inserted_post.id;
  let post_view = blocking(pool, move |conn| {
=======
  context: &LemmyContext,
) -> Result<HttpResponse, LemmyError> {
  let user = get_user_from_activity(&create, context).await?;
  let page = PageExt::from_any_base(create.object().to_owned().one().context(location_info!())?)?
    .context(location_info!())?;

  let post = PostForm::from_apub(&page, context, Some(user.actor_id()?)).await?;

  // Using an upsert, since likes (which fetch the post), sometimes come in before the create
  // resulting in double posts.
  let inserted_post = blocking(context.pool(), move |conn| Post::upsert(conn, &post)).await??;

  // Refetch the view
  let inserted_post_id = inserted_post.id;
  let post_view = blocking(context.pool(), move |conn| {
>>>>>>> 11149ba0
    PostView::read(conn, inserted_post_id, None)
  })
  .await??;

  let res = PostResponse { post: post_view };

<<<<<<< HEAD
  chat_server.do_send(SendPost {
    op: UserOperation::CreatePost,
    post: res,
    my_id: None,
  });

  announce_if_community_is_local(create, &user, client, pool).await?;
=======
  context.chat_server().do_send(SendPost {
    op: UserOperation::CreatePost,
    post: res,
    websocket_id: None,
  });

  announce_if_community_is_local(create, &user, context).await?;
>>>>>>> 11149ba0
  Ok(HttpResponse::Ok().finish())
}

async fn receive_create_comment(
  create: Create,
<<<<<<< HEAD
  client: &Client,
  pool: &DbPool,
  chat_server: ChatServerParam,
) -> Result<HttpResponse, LemmyError> {
  let user = get_user_from_activity(&create, client, pool).await?;
  let note = Note::from_any_base(create.object().to_owned().one().unwrap())?.unwrap();

  let comment = CommentForm::from_apub(&note, client, pool).await?;

  let inserted_comment = blocking(pool, move |conn| Comment::create(conn, &comment)).await??;

  let post_id = inserted_comment.post_id;
  let post = blocking(pool, move |conn| Post::read(conn, post_id)).await??;
=======
  context: &LemmyContext,
) -> Result<HttpResponse, LemmyError> {
  let user = get_user_from_activity(&create, context).await?;
  let note = Note::from_any_base(create.object().to_owned().one().context(location_info!())?)?
    .context(location_info!())?;

  let comment = CommentForm::from_apub(&note, context, Some(user.actor_id()?)).await?;

  let inserted_comment =
    blocking(context.pool(), move |conn| Comment::upsert(conn, &comment)).await??;

  let post_id = inserted_comment.post_id;
  let post = blocking(context.pool(), move |conn| Post::read(conn, post_id)).await??;
>>>>>>> 11149ba0

  // Note:
  // Although mentions could be gotten from the post tags (they are included there), or the ccs,
  // Its much easier to scrape them from the comment body, since the API has to do that
  // anyway.
  let mentions = scrape_text_for_mentions(&inserted_comment.content);
<<<<<<< HEAD
  let recipient_ids =
    send_local_notifs(mentions, inserted_comment.clone(), &user, post, pool, true).await?;

  // Refetch the view
  let comment_view = blocking(pool, move |conn| {
=======
  let recipient_ids = send_local_notifs(
    mentions,
    inserted_comment.clone(),
    &user,
    post,
    context.pool(),
    true,
  )
  .await?;

  // Refetch the view
  let comment_view = blocking(context.pool(), move |conn| {
>>>>>>> 11149ba0
    CommentView::read(conn, inserted_comment.id, None)
  })
  .await??;

  let res = CommentResponse {
    comment: comment_view,
    recipient_ids,
    form_id: None,
  };

<<<<<<< HEAD
  chat_server.do_send(SendComment {
    op: UserOperation::CreateComment,
    comment: res,
    my_id: None,
  });

  announce_if_community_is_local(create, &user, client, pool).await?;
=======
  context.chat_server().do_send(SendComment {
    op: UserOperation::CreateComment,
    comment: res,
    websocket_id: None,
  });

  announce_if_community_is_local(create, &user, context).await?;
>>>>>>> 11149ba0
  Ok(HttpResponse::Ok().finish())
}
