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
  comment_view::*,
  community_view::*,
  post::*,
  post_view::*,
  user::*,
  Crud,
  Reportable,
};
use serde::{Serialize, Deserialize};


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

#[async_trait::async_trait(?Send)]
impl Perform for Oper<CreateCommentReport> {
  type Response = CommentReportResponse;

  async fn perform(
    &self,
    pool: &DbPool,
    websocket_info: Option<WebsocketInfo>,
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
    let comment = blocking
      (pool, move |conn| CommentView::read(&conn, comment_id, None)).await??;

    // Check for community ban
    let is_banned =
      move |conn: &'_ _| CommunityUserBanView::get(conn, user_id,
						   comment.community_id).is_ok();
    if blocking(pool, is_banned).await? {
      return Err(APIError::err("community_ban").into());
    }

    // Insert the report
    let report_form = CommentReportForm {
      comment_id: comment_id,
      user_id: user_id,
      reason: data.reason.clone(),
      time: None, // column defaults to now() in table
      resolved: None, // columb defaults to false
    };
    blocking(pool, move |conn| CommentReport::report(conn, &report_form)).await??;      
    
    return Ok(CommentReportResponse{success:true});
  }
}

#[async_trait::async_trait(?Send)]
impl Perform for Oper<CreatePostReport> {
  type Response = PostReportResponse;

  async fn perform(
    &self,
    pool: &DbPool,
    websocket_info: Option<WebsocketInfo>,
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
    let post = blocking
      (pool, move |conn| PostView::read(&conn, post_id, None)).await??;

    // Check for community ban
    let is_banned =
      move |conn: &'_ _| CommunityUserBanView::get(conn, user_id,
						   post.community_id).is_ok();
    if blocking(pool, is_banned).await? {
      return Err(APIError::err("community_ban").into());
    }

    // Insert the report
    let report_form = PostReportForm {
      post_id: post_id,
      user_id: user_id,
      reason: data.reason.clone(),
      time: None, // column defaults to now() in table
      resolved: None, // columb defaults to false
    };
    blocking(pool, move |conn| PostReport::report(conn, &report_form)).await??;      
    
    return Ok(PostReportResponse{success:true});    
  }
}
