use lemmy_db::comment_view::CommentView;
use serde::{Deserialize, Serialize};
use lemmy_db::post_view::PostView;
use lemmy_db::community_view::{CommunityView, CommunityModeratorView};

#[derive(Serialize, Deserialize)]
pub struct GetComment {
  pub comment_id: i32,
  pub auth: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct GetCommentResponse {
  pub post: PostView,
  pub comments: Vec<CommentView>,
  pub community: CommunityView,
  pub moderators: Vec<CommunityModeratorView>,
}

#[derive(Serialize, Deserialize)]
pub struct CreateComment {
  pub content: String,
  pub parent_id: Option<i32>,
  pub post_id: i32,
  pub form_id: Option<String>,
  pub auth: String,
}

#[derive(Serialize, Deserialize)]
pub struct EditComment {
  pub content: String,
  pub edit_id: i32,
  pub form_id: Option<String>,
  pub auth: String,
}

#[derive(Serialize, Deserialize)]
pub struct DeleteComment {
  pub edit_id: i32,
  pub deleted: bool,
  pub auth: String,
}

#[derive(Serialize, Deserialize)]
pub struct RemoveComment {
  pub edit_id: i32,
  pub removed: bool,
  pub reason: Option<String>,
  pub auth: String,
}

#[derive(Serialize, Deserialize)]
pub struct MarkCommentAsRead {
  pub edit_id: i32,
  pub read: bool,
  pub auth: String,
}

#[derive(Serialize, Deserialize)]
pub struct SaveComment {
  pub comment_id: i32,
  pub save: bool,
  pub auth: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CommentResponse {
  pub comment: CommentView,
  pub recipient_ids: Vec<i32>,
  pub form_id: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct CreateCommentLike {
  pub comment_id: i32,
  pub score: i16,
  pub auth: String,
}

#[derive(Serialize, Deserialize)]
pub struct GetComments {
  pub type_: String,
  pub sort: String,
  pub page: Option<i64>,
  pub limit: Option<i64>,
  pub community_id: Option<i32>,
  pub auth: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct GetCommentsResponse {
  pub comments: Vec<CommentView>,
}
