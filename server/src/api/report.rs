use crate::{
  api::{claims::Claims, APIError, Oper, Perform},
  blocking,
  websocket::WebsocketInfo,
  DbPool, LemmyError,
};
use lemmy_db::{
  comment::*,
  comment_view::*,
  community_view::*,
  post::*,
  post_view::*,
  report_views::{
    CommentReportQueryBuilder, CommentReportView, PostReportQueryBuilder, PostReportView,
  },
  user::*,
  user_view::UserView,
  Crud, Reportable,
};

use serde::{Deserialize, Serialize};

const MAX_REPORT_LEN: usize = 1000;

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
pub struct ListCommentReports {
  page: Option<i64>,
  limit: Option<i64>,
  pub community: i32,
  auth: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ListCommentReportResponse {
  pub reports: Vec<CommentReportView>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ListPostReports {
  page: Option<i64>,
  limit: Option<i64>,
  pub community: i32,
  auth: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ListPostReportResponse {
  pub reports: Vec<PostReportView>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetReportCount {
  community: i32,
  auth: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetReportCountResponse {
  community: i32,
  comment_reports: usize,
  post_reports: usize,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ResolveCommentReport {
  pub report: uuid::Uuid,
  pub auth: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ResolveCommentReportResponse {
  pub report: uuid::Uuid,
  pub resolved: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ResolvePostReport {
  pub report: uuid::Uuid,
  pub auth: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ResolvePostReportResponse {
  pub report: uuid::Uuid,
  pub resolved: bool,
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

    // Check size of report and check for whitespace
    let reason: Option<String> = match data.reason.clone() {
      Some(s) if s.trim().is_empty() => None,
      Some(s) if s.len() > MAX_REPORT_LEN => {
        return Err(APIError::err("report_too_long").into());
      }
      Some(s) => Some(s),
      None => None,
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
    let community_id = comment.community_id;
    let is_banned =
      move |conn: &'_ _| CommunityUserBanView::get(conn, user_id, community_id).is_ok();
    if blocking(pool, is_banned).await? {
      return Err(APIError::err("community_ban").into());
    }

    // Insert the report
    let comment_time = match comment.updated {
      Some(s) => s,
      None => comment.published,
    };
    let report_form = CommentReportForm {
      time: None, // column defaults to now() in table
      reason,
      resolved: None, // column defaults to false
      user_id,
      comment_id,
      comment_text: comment.content,
      comment_time,
    };
    blocking(pool, move |conn| CommentReport::report(conn, &report_form)).await??;

    Ok(CommentReportResponse { success: true })
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

    // Check size of report and check for whitespace
    let reason: Option<String> = match data.reason.clone() {
      Some(s) if s.trim().is_empty() => None,
      Some(s) if s.len() > MAX_REPORT_LEN => {
        return Err(APIError::err("report_too_long").into());
      }
      Some(s) => Some(s),
      None => None,
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
    let community_id = post.community_id;
    let is_banned =
      move |conn: &'_ _| CommunityUserBanView::get(conn, user_id, community_id).is_ok();
    if blocking(pool, is_banned).await? {
      return Err(APIError::err("community_ban").into());
    }

    // Insert the report
    let post_time = match post.updated {
      Some(s) => s,
      None => post.published,
    };
    let report_form = PostReportForm {
      time: None, // column defaults to now() in table
      reason,
      resolved: None, // columb defaults to false
      user_id,
      post_id,
      post_name: post.name,
      post_url: post.url,
      post_body: post.body,
      post_time,
    };
    blocking(pool, move |conn| PostReport::report(conn, &report_form)).await??;

    Ok(PostReportResponse { success: true })
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for Oper<GetReportCount> {
  type Response = GetReportCountResponse;

  async fn perform(
    &self,
    pool: &DbPool,
    _websocket_info: Option<WebsocketInfo>,
  ) -> Result<GetReportCountResponse, LemmyError> {
    let data: &GetReportCount = &self.data;

    let claims = match Claims::decode(&data.auth) {
      Ok(claims) => claims.claims,
      Err(_e) => return Err(APIError::err("not_logged_in").into()),
    };

    let user_id = claims.id;
    let user = blocking(pool, move |conn| User_::read(&conn, user_id)).await??;
    if user.banned {
      return Err(APIError::err("site_ban").into());
    }

    let community_id = data.community;
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
      return Err(APIError::err("report_view_not_allowed").into());
    }

    let comment_reports = blocking(pool, move |conn| {
      CommentReportQueryBuilder::create(conn)
        .community_id(community_id)
        .resolved(false)
        .count()
    })
    .await??;
    let post_reports = blocking(pool, move |conn| {
      PostReportQueryBuilder::create(conn)
        .community_id(community_id)
        .resolved(false)
        .count()
    })
    .await??;

    let response = GetReportCountResponse {
      community: community_id,
      comment_reports,
      post_reports,
    };

    Ok(response)
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

    let community_id = data.community;
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
      return Err(APIError::err("report_view_not_allowed").into());
    }

    let page = data.page;
    let limit = data.limit;
    let reports = blocking(pool, move |conn| {
      CommentReportQueryBuilder::create(conn)
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

    let community_id = data.community;
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
      return Err(APIError::err("report_view_not_allowed").into());
    }

    let page = data.page;
    let limit = data.limit;
    let reports = blocking(pool, move |conn| {
      PostReportQueryBuilder::create(conn)
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
impl Perform for Oper<ResolveCommentReport> {
  type Response = ResolveCommentReportResponse;

  async fn perform(
    &self,
    pool: &DbPool,
    _websocket_info: Option<WebsocketInfo>,
  ) -> Result<ResolveCommentReportResponse, LemmyError> {
    let data: &ResolveCommentReport = &self.data;

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

    // Fetch the report view
    let report_id = data.report;
    let report = blocking(pool, move |conn| CommentReportView::read(&conn, &report_id)).await??;

    // Check for community ban
    let community_id = report.community_id;
    let is_banned =
      move |conn: &'_ _| CommunityUserBanView::get(conn, user_id, community_id).is_ok();
    if blocking(pool, is_banned).await? {
      return Err(APIError::err("community_ban").into());
    }

    // Check for mod/admin privileges
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
      return Err(APIError::err("resolve_report_not_allowed").into());
    }

    blocking(pool, move |conn| {
      CommentReport::resolve(conn, &report_id.clone())
    })
    .await??;

    Ok(ResolveCommentReportResponse {
      report: report_id,
      resolved: true,
    })
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for Oper<ResolvePostReport> {
  type Response = ResolvePostReportResponse;

  async fn perform(
    &self,
    pool: &DbPool,
    _websocket_info: Option<WebsocketInfo>,
  ) -> Result<ResolvePostReportResponse, LemmyError> {
    let data: &ResolvePostReport = &self.data;

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

    // Fetch the report view
    let report_id = data.report;
    let report = blocking(pool, move |conn| PostReportView::read(&conn, &report_id)).await??;

    // Check for community ban
    let community_id = report.community_id;
    let is_banned =
      move |conn: &'_ _| CommunityUserBanView::get(conn, user_id, community_id).is_ok();
    if blocking(pool, is_banned).await? {
      return Err(APIError::err("community_ban").into());
    }

    // Check for mod/admin privileges
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
      return Err(APIError::err("resolve_report_not_allowed").into());
    }

    blocking(pool, move |conn| {
      PostReport::resolve(conn, &report_id.clone())
    })
    .await??;

    Ok(ResolvePostReportResponse {
      report: report_id,
      resolved: true,
    })
  }
}
