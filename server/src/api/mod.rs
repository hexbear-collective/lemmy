use actix_web::web::Data;

use lemmy_api_structs::APIError;
use lemmy_db::{
  community::Community, community_view::CommunityUserBanView, naive_now, post::Post, user::User_,
  Crud,
};
use lemmy_utils::{settings::Settings, slur_check, slurs_vec_to_str, ConnectionId, LemmyError};

use crate::{api::claims::Claims, blocking, DbPool, LemmyContext};
use chrono::Duration;
use lemmy_db::user_ban_id::UserBanId;
use lemmy_db::user_token::UserToken;

pub mod claims;
pub mod comment;
pub mod community;
pub mod community_settings;
pub mod post;
pub mod post_hexbear;
pub mod report;
pub mod site;
pub mod user;

#[async_trait::async_trait(?Send)]
pub trait Perform {
  type Response: serde::ser::Serialize + Send;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    websocket_id: Option<ConnectionId>,
  ) -> Result<Self::Response, LemmyError>;
}

pub(in crate::api) async fn is_mod_or_admin(
  pool: &DbPool,
  user_id: i32,
  community_id: i32,
) -> Result<(), LemmyError> {
  let is_mod_or_admin = blocking(pool, move |conn| {
    Community::is_mod_or_admin(conn, user_id, community_id)
  })
  .await?;
  if !is_mod_or_admin {
    return Err(APIError::err("not_a_mod_or_admin").into());
  }
  Ok(())
}
pub async fn is_admin(pool: &DbPool, user_id: i32) -> Result<(), LemmyError> {
  let user = blocking(pool, move |conn| User_::read(conn, user_id)).await??;
  if !user.admin {
    return Err(APIError::err("not_an_admin").into());
  }
  Ok(())
}

pub async fn is_admin_or_sitemod(pool: &DbPool, user_id: i32) -> Result<(), LemmyError> {
  let user = blocking(pool, move |conn| User_::read(conn, user_id)).await??;
  if !(user.admin || user.sitemod) {
    return Err(APIError::err("not_an_admin").into());
  }
  Ok(())
}

pub(in crate::api) async fn get_post(post_id: i32, pool: &DbPool) -> Result<Post, LemmyError> {
  match blocking(pool, move |conn| Post::read(conn, post_id)).await? {
    Ok(post) => Ok(post),
    Err(_e) => Err(APIError::err("couldnt_find_post").into()),
  }
}

pub(in crate::api) async fn get_user_from_jwt(
  jwt: &str,
  pool: &DbPool,
) -> Result<User_, LemmyError> {
  let jwt_split = jwt.split(":").collect::<Vec<_>>();
  let mut bid_string = jwt_split.get(1).unwrap_or(&"").to_string();

  let claims = match Claims::decode(&jwt_split[0]) {
    Ok(claims) => claims.claims,
    Err(_e) => return Err(APIError::err("not_logged_in").into()),
  };

  validate_token(claims.token_id, pool).await?;

  let user_id = claims.id;
  let user = blocking(pool, move |conn| User_::read(conn, user_id)).await??;

  if !bid_string.is_empty() {
    //bid reported, try creating relationship
    let bid = bid_string
      .parse()
      .map_err(|_| APIError::err("invalid_bid"))?;
    blocking(pool, move |conn| UserBanId::associate(conn, bid, user_id)).await??;
  } else {
    //bid not reported, find existing
    bid_string = match blocking(pool, move |conn| UserBanId::get_by_user(conn, &user_id)).await? {
      Ok(Some(ubid)) => ubid.bid.to_string(),
      Ok(None) => "".to_string(),
      //another error
      Err(_) => return Err(APIError::err("internal_error").into()),
    }
  }

  // Check for a site ban
  if user.banned {
    //generate new bid
    if bid_string.is_empty() {
      bid_string = blocking(pool, move |conn| {
        UserBanId::create_then_associate(conn, user_id.clone())
      })
      .await??
      .bid
      .to_string();
    }

    return Err(APIError::err(&*format!("site_ban_{}", bid_string)).into());
  }
  Ok(user)
}

pub(in crate::api) async fn validate_token(
  token_id: uuid::Uuid,
  pool: &DbPool,
) -> Result<(), LemmyError> {
  let token = match blocking(pool, move |conn| UserToken::read(conn, token_id)).await? {
    Ok(user_token) => user_token,
    Err(_e) => return Err(APIError::err("not_logged_in").into()),
  };

  if token.is_revoked {
    return Err(APIError::err("not_logged_in").into());
  }

  let settings = Settings::get();

  let time_to_refresh =
    naive_now() + Duration::minutes(settings.auth_token.renew_window_minutes.into());
  if token.expires_at < time_to_refresh {
    blocking(pool, move |conn| {
      UserToken::renew(conn, token.id, settings.auth_token.renew_minutes.into())
    })
    .await??;
  }

  Ok(())
}

pub(in crate::api) async fn get_user_from_jwt_opt(
  jwt: &Option<String>,
  pool: &DbPool,
) -> Result<Option<User_>, LemmyError> {
  match jwt {
    Some(jwt) => Ok(Some(get_user_from_jwt(jwt, pool).await?)),
    None => Ok(None),
  }
}

pub(in crate) fn check_slurs(text: &str) -> Result<(), APIError> {
  if let Err(slurs) = slur_check(text) {
    Err(APIError::err(&slurs_vec_to_str(slurs)))
  } else {
    Ok(())
  }
}
pub(in crate) fn check_slurs_opt(text: &Option<String>) -> Result<(), APIError> {
  match text {
    Some(t) => check_slurs(t),
    None => Ok(()),
  }
}
pub(in crate::api) async fn check_community_ban(
  user_id: i32,
  community_id: i32,
  pool: &DbPool,
) -> Result<(), LemmyError> {
  let is_banned = move |conn: &'_ _| CommunityUserBanView::get(conn, user_id, community_id).is_ok();
  if blocking(pool, is_banned).await? {
    Err(APIError::err("community_ban").into())
  } else {
    Ok(())
  }
}
