use crate::{api::Perform, LemmyContext};
use actix_web::{error::ErrorBadRequest, *};
use lemmy_api_structs::{
  comment::*,
  community::*,
  community_settings::*,
  post::*,
  post_hexbear::{FeaturePost, GetFeaturedPosts},
  report::*,
  site::*,
  user::*,
};
use lemmy_rate_limit::RateLimit;
use serde::Serialize;

pub fn config(cfg: &mut web::ServiceConfig, rate_limit: &RateLimit) {
  cfg.service(
    web::scope("/api/v1")
      // Websockets
      .service(web::resource("/ws").to(super::websocket::chat_route))
      // Site
      .service(
        web::scope("/site")
          .wrap(rate_limit.message())
          .route("", web::get().to(route_get::<GetSite>))
          // Admin Actions
          .route("", web::post().to(route_post::<CreateSite>))
          .route("", web::put().to(route_post::<EditSite>))
          .route("/transfer", web::post().to(route_post::<TransferSite>))
          .route("/config", web::get().to(route_get::<GetSiteConfig>))
          .route("/config", web::put().to(route_post::<SaveSiteConfig>))
          .route("/mods", web::get().to(route_get::<GetSiteModerators>)),
      )
      .service(
        web::resource("/categories")
          .wrap(rate_limit.message())
          .route(web::get().to(route_get::<ListCategories>)),
      )
      .service(
        web::resource("/modlog")
          .wrap(rate_limit.message())
          .route(web::get().to(route_get::<GetModlog>)),
      )
      .service(
        web::resource("/search")
          .wrap(rate_limit.message())
          .route(web::get().to(route_get::<Search>)),
      )
      // Community
      .service(
        web::resource("/community")
          .guard(guard::Post())
          .wrap(rate_limit.register())
          .route(web::post().to(route_post::<CreateCommunity>)),
      )
      .service(
        web::scope("/community")
          .wrap(rate_limit.message())
          .route("", web::get().to(route_get::<GetCommunity>))
          .route("", web::put().to(route_post::<EditCommunity>))
          .route("/list", web::get().to(route_get::<ListCommunities>))
          .route("/follow", web::post().to(route_post::<FollowCommunity>))
          .route("/delete", web::post().to(route_post::<DeleteCommunity>))
          // Mod Actions
          .route("/remove", web::post().to(route_post::<RemoveCommunity>))
          .route("/transfer", web::post().to(route_post::<TransferCommunity>))
          .route("/ban_user", web::post().to(route_post::<BanFromCommunity>))
          .route("/mod", web::post().to(route_post::<AddModToCommunity>))
          .route(
            "/settings",
            web::get().to(route_get::<GetCommunitySettings>),
          )
          .route(
            "/settings",
            web::put().to(route_post::<EditCommunitySettings>),
          )
          .route(
            "/comment_reports",
            web::get().to(route_get::<ListCommentReports>),
          )
          .route("/post_reports", web::get().to(route_get::<ListPostReports>))
          .route("/reports", web::get().to(route_get::<GetReportCount>)),
      )
      // Post
      .service(
        // Handle POST to /post separately to add the post() rate limitter
        web::resource("/post")
          .guard(guard::Post())
          .wrap(rate_limit.post())
          .route(web::post().to(route_post::<CreatePost>)),
      )
      .service(
        // Handle POST to /post/report separately to add the post() rate limitter
        web::resource("/post/report")
          .wrap(rate_limit.report())
          .route(web::post().to(route_post::<CreatePostReport>)),
      )
      .service(
        web::scope("/post")
          .wrap(rate_limit.message())
          .route("", web::get().to(route_get::<GetPost>))
          .route("", web::put().to(route_post::<EditPost>))
          .route("/delete", web::post().to(route_post::<DeletePost>))
          .route("/remove", web::post().to(route_post::<RemovePost>))
          .route("/lock", web::post().to(route_post::<LockPost>))
          .route("/sticky", web::post().to(route_post::<StickyPost>))
          .route("/list", web::get().to(route_get::<GetPosts>))
          .route("/like", web::post().to(route_post::<CreatePostLike>))
          .route("/save", web::put().to(route_post::<SavePost>))
          .route(
            "/resolve_report",
            web::post().to(route_post::<ResolvePostReport>),
          )
          .route("/featured", web::get().to(route_get::<GetFeaturedPosts>))
          .route("/featured", web::put().to(route_post::<FeaturePost>)),
      )
      // Comment
      .service(
        // Handle POST to /comment separately to add the post() rate limitter
        web::resource("/comment")
          .guard(guard::Post())
          .wrap(rate_limit.comment())
          .route(web::post().to(route_post::<CreateComment>)),
      )
      .service(
        // Handle POST to /comment/report separately to add the post() rate limitter
        web::resource("/comment/report")
          .wrap(rate_limit.report())
          .route(web::post().to(route_post::<CreateCommentReport>)),
      )
      .service(
        web::scope("/comment")
          .wrap(rate_limit.message())
          .route("", web::put().to(route_post::<EditComment>))
          .route("/delete", web::post().to(route_post::<DeleteComment>))
          .route("/remove", web::post().to(route_post::<RemoveComment>))
          .route(
            "/mark_as_read",
            web::post().to(route_post::<MarkCommentAsRead>),
          )
          .route("/like", web::post().to(route_post::<CreateCommentLike>))
          .route("/save", web::put().to(route_post::<SaveComment>))
          .route(
            "/resolve_report",
            web::post().to(route_post::<ResolveCommentReport>),
          )
          .route("", web::get().to(route_get::<GetComment>))
          .route("/list", web::get().to(route_get::<GetComments>)),
      )
      // Private Message
      .service(
        web::resource("/private_message")
          .guard(guard::Post())
          .wrap(rate_limit.direct_message())
          .route(web::post().to(route_post::<CreatePrivateMessage>)),
      )
      .service(
        web::scope("/private_message")
          .wrap(rate_limit.message())
          .route("/list", web::get().to(route_get::<GetPrivateMessages>))
          .route("", web::post().to(route_post::<CreatePrivateMessage>))
          .route("", web::put().to(route_post::<EditPrivateMessage>))
          .route(
            "/delete",
            web::post().to(route_post::<DeletePrivateMessage>),
          )
          .route(
            "/mark_as_read",
            web::post().to(route_post::<MarkPrivateMessageAsRead>),
          ),
      )
      // User
      .service(
        // Account action, I don't like that it's in /user maybe /accounts
        // Handle /user/register separately to add the register() rate limitter
        web::resource("/user/register")
          .guard(guard::Post())
          .wrap(rate_limit.register())
          .route(web::post().to(route_post::<Register>)),
      )
      // User actions
      .service(
        web::scope("/user")
          .wrap(rate_limit.message())
          .route("", web::get().to(route_get::<GetUserDetails>))
          .route("/mention", web::get().to(route_get::<GetUserMentions>))
          .route(
            "/mention/mark_as_read",
            web::post().to(route_post::<MarkUserMentionAsRead>),
          )
          .route("/replies", web::get().to(route_get::<GetReplies>))
          .route(
            "/followed_communities",
            web::get().to(route_get::<GetFollowedCommunities>),
          )
          .route("/join", web::post().to(route_post::<UserJoin>))
          // Admin action. I don't like that it's in /user
          .route("/ban", web::post().to(route_post::<BanUser>))
          .route("/purge", web::post().to(route_post::<RemoveUserContent>))
          .route("/relations", web::get().to(route_get::<GetRelatedUsers>))
          // Account actions. I don't like that they're in /user maybe /accounts
          .route("/login", web::post().to(route_post::<Login>))
          .route("/logout", web::post().to(route_post::<Logout>))
          .route("/get_captcha", web::get().to(route_post::<GetCaptcha>))
          .route(
            "/delete_account",
            web::post().to(route_post::<DeleteAccount>),
          )
          .route(
            "/password_reset",
            web::post().to(route_post::<PasswordReset>),
          )
          .route(
            "/password_change",
            web::post().to(route_post::<PasswordChange>),
          )
          // mark_all_as_read feels off being in this section as well
          .route(
            "/mark_all_as_read",
            web::post().to(route_post::<MarkAllAsRead>),
          )
          .route(
            "/save_user_settings",
            web::put().to(route_post::<SaveUserSettings>),
          )
          .route("/tags", web::get().to(route_get::<GetUserTag>))
          .route("/tags", web::post().to(route_post::<SetUserTag>))
          .route("/unread_notifs", web::get().to(route_get::<GetUnreadCount>)),
      )
      // Admin Actions
      .service(
        web::resource("/admin/add")
          .wrap(rate_limit.message())
          .route(web::post().to(route_post::<AddAdmin>)),
      )
      // Sitemod Actions
      .service(
        web::resource("/sitemod/add")
          .wrap(rate_limit.message())
          .route(web::post().to(route_post::<AddSitemod>)),
      ),
  );
}

async fn perform<Request>(
  data: Request,
  context: web::Data<LemmyContext>,
) -> Result<HttpResponse, Error>
where
  Request: Perform,
  Request: Send + 'static,
{
  let res = data
    .perform(&context, None)
    .await
    .map(|json| HttpResponse::Ok().json(json))
    .map_err(ErrorBadRequest)?;
  Ok(res)
}

async fn route_get<Data>(
  data_query: Result<web::Query<Data>>,
  data_json: Result<web::Json<Data>>,
  context: web::Data<LemmyContext>,
) -> Result<HttpResponse, Error>
where
  Data: Serialize + Send + 'static + Perform,
{
  match data_json {
    Ok(data_j) => perform::<Data>(data_j.0, context).await,
    Err(_) => match data_query {
      Ok(data_q) => perform::<Data>(data_q.0, context).await,
      Err(err_q) => Err(err_q),
    },
  }
}

async fn route_post<Data>(
  data_json: Result<web::Json<Data>>,
  data_form: Result<web::Form<Data>>,
  context: web::Data<LemmyContext>,
) -> Result<HttpResponse, Error>
where
  Data: Serialize + Send + 'static + Perform,
{
  match data_json {
    Ok(data_j) => perform::<Data>(data_j.0, context).await,
    Err(err_j) => match data_form {
      Ok(data_f) => perform::<Data>(data_f.0, context).await,
      Err(_) => Err(err_j),
    },
  }
}
