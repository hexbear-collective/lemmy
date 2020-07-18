use crate::{
  api::{claims::Claims, APIError, Oper, Perform},
  blocking,
  websocket::{
    server::{JoinCommunityRoom, SendComment},
    UserOperation,
    WebsocketInfo,
  },
  DbPool,
  LemmyError,
};
use lemmy_db::{
  comment::*,
  comment_report_view::{CommentReportView, CommentReportViewQueryBuilder},
  comment_view::*,
  community_view::*,
  post::*,
  post_report_view::{PostReportView, PostReportViewQueryBuilder},
  post_view::*,
  user::*,
  user_view::UserView,
  Crud,
  Reportable,
};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct CreateCommentReport {
  comment: i32,
  reason: Option<String>,
  auth: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CommentReportResponse {
  pub success: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ListCommentReportResponse {
  pub reports: Vec<CommentReportView>,
}

#[derive(Serialize, Deserialize)]
pub struct CreatePostReport {
  post: i32,
  reason: Option<String>,
  auth: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PostReportResponse {
  pub success: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ListPostReports {
  page: Option<i64>,
  limit: Option<i64>,
  pub community_id: i32,
  auth: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ListCommentReports {
  page: Option<i64>,
  limit: Option<i64>,
  pub community_id: i32,
  auth: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ListPostReportResponse {
  pub reports: Vec<PostReportView>,
}

#[async_trait::async_trait(?Send)]
impl Perform for Oper<CreateCommentReport> {
  type Response = CommentReportResponse;

  async fn perform(
    &self,
    pool: &DbPool,
    _websocket_info: Option<WebsocketInfo>,
  ) -> Result<CommentReportResponse, LemmyError> {
    let data: &CreateCommentReport = &self.data;

    // Verify auth token
    let claims = match Claims::decode(&data.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => return Err(APIError::err("not_logged_in").into()),
    };

    // Check for site ban
    let user_id = claims.id;
    let user = blocking(pool, move |conn| User_::read(&conn, user_id)).await??;
    if user.banned {
      return Err(APIError::err("site_ban").into());
    }

    // Fetch comment information
    let comment_id = data.comment;
    let comment = blocking(pool, move |conn| CommentView::read(&conn, comment_id, None)).await??;

    // Check for community ban
    let is_banned =
      move |conn: &'_ _| CommunityUserBanView::get(conn, user_id, comment.community_id).is_ok();
    if blocking(pool, is_banned).await? {
      return Err(APIError::err("community_ban").into());
    }

    // Insert the report
    let report_form = CommentReportForm {
      comment_id,
      user_id,
      reason: data.reason.clone(),
      time: None,     // column defaults to now() in table
      resolved: None, // columb defaults to false
    };
    blocking(pool, move |conn| CommentReport::report(conn, &report_form)).await??;

    Ok(CommentReportResponse { success: true })
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for Oper<ListCommentReports> {
  type Response = ListCommentReportResponse;

  async fn perform(
    &self,
    pool: &DbPool,
    _websocket_info: Option<WebsocketInfo>,
  ) -> Result<ListCommentReportResponse, LemmyError> {
    let data: &ListCommentReports = &self.data;

    let claims = match Claims::decode(&data.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => return Err(APIError::err("not_logged_in").into()),
    };

    let user_id = claims.id;
    let user = blocking(pool, move |conn| User_::read(&conn, user_id)).await??;
    if user.banned {
      return Err(APIError::err("site_ban").into());
    }

    let community_id = data.community_id;
    //Check community exists.
    let community_id = blocking(pool, move |conn| {
      CommunityView::read(conn, community_id, None)
    })
    .await??
    .id;
    // Check for community ban
    let is_banned =
      move |conn: &'_ _| CommunityUserBanView::get(conn, user_id, community_id).is_ok();
    if blocking(pool, is_banned).await? {
      return Err(APIError::err("community_ban").into());
    }

    let mut mod_ids: Vec<i32> = Vec::new();
    mod_ids.append(
      &mut blocking(pool, move |conn| {
        CommunityModeratorView::for_community(conn, community_id)
          .map(|v| v.into_iter().map(|m| m.user_id).collect())
      })
      .await??,
    );
    mod_ids.append(
      &mut blocking(pool, move |conn| {
        UserView::admins(conn).map(|v| v.into_iter().map(|a| a.id).collect())
      })
      .await??,
    );
    if !mod_ids.contains(&user_id) {
      return Err(APIError::err("no_view_reports_allowed").into());
    }

    let page = data.page;
    let limit = data.limit;
    let reports = blocking(pool, move |conn| {
      CommentReportViewQueryBuilder::create(conn)
        .community_id(community_id)
        .page(page)
        .limit(limit)
        .list()
    })
    .await??;

    Ok(ListCommentReportResponse { reports })
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for Oper<ListPostReports> {
  type Response = ListPostReportResponse;

  async fn perform(
    &self,
    pool: &DbPool,
    _websocket_info: Option<WebsocketInfo>,
  ) -> Result<ListPostReportResponse, LemmyError> {
    let data: &ListPostReports = &self.data;

    // Verify auth token
    let claims = match Claims::decode(&data.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => return Err(APIError::err("not_logged_in").into()),
    };

    let user_id = claims.id;
    let user = blocking(pool, move |conn| User_::read(&conn, user_id)).await??;
    if user.banned {
      return Err(APIError::err("site_ban").into());
    }

    let community_id = data.community_id;
    //Check community exists.
    let community_id = blocking(pool, move |conn| {
      CommunityView::read(conn, community_id, None)
    })
    .await??
    .id;
    // Check for community ban
    let is_banned =
      move |conn: &'_ _| CommunityUserBanView::get(conn, user_id, community_id).is_ok();
    if blocking(pool, is_banned).await? {
      return Err(APIError::err("community_ban").into());
    }

    let mut mod_ids: Vec<i32> = Vec::new();
    mod_ids.append(
      &mut blocking(pool, move |conn| {
        CommunityModeratorView::for_community(conn, community_id)
          .map(|v| v.into_iter().map(|m| m.user_id).collect())
      })
      .await??,
    );
    mod_ids.append(
      &mut blocking(pool, move |conn| {
        UserView::admins(conn).map(|v| v.into_iter().map(|a| a.id).collect())
      })
      .await??,
    );
    if !mod_ids.contains(&user_id) {
      return Err(APIError::err("no_view_reports_allowed").into());
    }

    let page = data.page;
    let limit = data.limit;
    let reports = blocking(pool, move |conn| {
      PostReportViewQueryBuilder::create(conn)
        .community_id(community_id)
        .page(page)
        .limit(limit)
        .list()
    })
    .await??;

    Ok(ListPostReportResponse { reports })
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for Oper<CreatePostReport> {
  type Response = PostReportResponse;

  async fn perform(
    &self,
    pool: &DbPool,
    _websocket_info: Option<WebsocketInfo>,
  ) -> Result<PostReportResponse, LemmyError> {
    let data: &CreatePostReport = &self.data;

    // Verify auth token
    let claims = match Claims::decode(&data.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => return Err(APIError::err("not_logged_in").into()),
    };

    // Check for site ban
    let user_id = claims.id;
    let user = blocking(pool, move |conn| User_::read(&conn, user_id)).await??;
    if user.banned {
      return Err(APIError::err("site_ban").into());
    }

    // Fetch post information from the database
    let post_id = data.post;
    let post = blocking(pool, move |conn| PostView::read(&conn, post_id, None)).await??;

    // Check for community ban
    let is_banned =
      move |conn: &'_ _| CommunityUserBanView::get(conn, user_id, post.community_id).is_ok();
    if blocking(pool, is_banned).await? {
      return Err(APIError::err("community_ban").into());
    }

    // Insert the report
    let report_form = PostReportForm {
      post_id,
      user_id,
      reason: data.reason.clone(),
      time: None,     // column defaults to now() in table
      resolved: None, // columb defaults to false
    };
    blocking(pool, move |conn| PostReport::report(conn, &report_form)).await??;

    Ok(PostReportResponse { success: true })
  }
}
