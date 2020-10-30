use actix_web::web::Data;

use lemmy_api_structs::{
  post::*,
  post_hexbear::{FeaturePost, GetFeaturedPosts},
  APIError,
};
use lemmy_db::{post::*, post_view::*};
use lemmy_utils::{ConnectionId, LemmyError};

use crate::{
  api::{get_user_from_jwt, get_user_from_jwt_opt, is_admin_or_sitemod, Perform},
  blocking,
  websocket::{messages::SendPost, UserOperation},
  LemmyContext,
};

#[async_trait::async_trait(?Send)]
impl Perform for GetFeaturedPosts {
  type Response = GetPostsResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<GetPostsResponse, LemmyError> {
    let data: &GetFeaturedPosts = &self;
    let user = get_user_from_jwt_opt(&data.auth, context.pool()).await?;

    let user_id = match &user {
      Some(user) => Some(user.id),
      None => None,
    };

    let posts = match blocking(context.pool(), move |conn| {
      PostQueryBuilder::create(conn)
        .my_user_id(user_id)
        .featured(true)
        .limit(2)
        .list()
    })
    .await?
    {
      Ok(posts) => posts,
      Err(_e) => return Err(APIError::err("couldnt_get_posts").into()),
    };

    Ok(GetPostsResponse { posts })
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for FeaturePost {
  type Response = PostResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    websocket_id: Option<ConnectionId>,
  ) -> Result<PostResponse, LemmyError> {
    let data: &FeaturePost = &self;

    let user = get_user_from_jwt(&data.auth, context.pool()).await?;

    is_admin_or_sitemod(context.pool(), user.id).await?;

    let featured = data.featured;
    if featured {
      match blocking(context.pool(), move |conn| {
        PostQueryBuilder::create(conn).featured(true).list()
      })
      .await?
      {
        Ok(posts) => {
          if posts.len() >= 2 {
            return Err(APIError::err("max_posts_featured").into());
          }
        }
        Err(_e) => {}
      };
    }

    let post_id = data.id;
    let featured = data.featured;
    blocking(context.pool(), move |conn| {
      Post::update_featured(conn, post_id, featured)
    })
    .await??;

    let post_view = blocking(context.pool(), move |conn| {
      PostView::read(conn, post_id, Some(user.id))
    })
    .await??;

    let res = PostResponse { post: post_view };

    context.chat_server().do_send(SendPost {
      op: UserOperation::FeaturePost,
      post: res.clone(),
      websocket_id,
    });

    Ok(res)
  }
}
