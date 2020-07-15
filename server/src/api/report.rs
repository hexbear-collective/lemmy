use crate::{
  api::{APIError, Oper, Perform},
  blocking,
  db::{
    comment::*,
    comment_view::*,
    post::*,
    post_view::*,
    user::*,
  },
  websocket::{
    server::{JoinCommunityRoom, SendComment},
    UserOperation,
    WebsocketInfo,
  },
  DbPool,
  LemmyError,
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
  pub comment: CommentView,
}

#[derive(Serialize, Deserialize)]
pub struct CreatePostReport {
  post: i32,
  reason: Option<String>,
  auth: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PostReportResponse {
  pub post: PostView,
}

#[async_trait::async_trait(?Send)]
impl Perform for Oper<CreateCommentReport> {
  type Response = CommentReportResponse;

  async fn perform(
    &self,
    pool: &DbPool,
    websocket_info: Option<WebsocketInfo>,
  ) -> Result<CommentReportResponse, LemmyError> {
    return Err(APIError::err("comment_report_not_implemented").into());
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
    return Err(APIError::err("post_report_not_implemented").into());
  }
}
