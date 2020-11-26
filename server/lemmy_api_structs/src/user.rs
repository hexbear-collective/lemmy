use lemmy_db::{
  comment_view::{CommentView, ReplyView},
  community_view::{CommunityFollowerView, CommunityModeratorView},
  post_view::PostView,
  private_message_view::PrivateMessageView,
  user_mention_view::UserMentionView,
  user_view::UserView,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Login {
  pub username_or_email: String,
  pub password: String,
  pub code_2fa: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Logout {
  pub auth: String,
}

#[derive(Serialize, Deserialize)]
pub struct Register {
  pub username: String,
  pub email: Option<String>,
  pub password: String,
  pub password_verify: String,
  pub admin: bool,
  pub show_nsfw: bool,
  pub captcha_uuid: Option<String>,
  pub captcha_answer: Option<String>,
  pub pronouns: Option<String>,    // hexbear
  pub hcaptcha_id: Option<String>, // hexbear
}

#[derive(Serialize, Deserialize)]
pub struct GetCaptcha {}

#[derive(Serialize, Deserialize)]
pub struct GetCaptchaResponse {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub ok: Option<CaptchaResponse>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub hcaptcha: Option<HCaptchaResponse>,
}

#[derive(Serialize, Deserialize)]
pub struct CaptchaResponse {
  pub png: String,         // A Base64 encoded png
  pub wav: Option<String>, // A Base64 encoded wav audio
  pub uuid: String,
}

#[derive(Serialize, Deserialize)]
pub struct SaveUserSettings {
  pub show_nsfw: bool,
  pub theme: String,
  pub default_sort_type: i16,
  pub default_listing_type: i16,
  pub lang: String,
  pub avatar: Option<String>,
  pub banner: Option<String>,
  pub preferred_username: Option<String>,
  pub email: Option<String>,
  pub bio: Option<String>,
  pub matrix_user_id: Option<String>,
  pub new_password: Option<String>,
  pub new_password_verify: Option<String>,
  pub old_password: Option<String>,
  pub show_avatars: bool,
  pub send_notifications_to_email: bool,
  pub has_2fa: bool,
  pub auth: String,
  pub inbox_disabled: bool,
}

#[derive(Serialize, Deserialize)]
pub struct LoginResponse {
  pub requires_2fa: bool, //this should be exclusive with jwt
  pub jwt: String,
}

#[derive(Serialize, Deserialize)]
pub struct GetUserDetails {
  pub user_id: Option<i32>,
  pub username: Option<String>,
  pub sort: String,
  pub page: Option<i64>,
  pub limit: Option<i64>,
  pub community_id: Option<i32>,
  pub saved_only: bool,
  pub auth: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct GetUserDetailsResponse {
  pub user: UserView,
  pub follows: Vec<CommunityFollowerView>,
  pub moderates: Vec<CommunityModeratorView>,
  pub comments: Vec<CommentView>,
  pub posts: Vec<PostView>,
  // TODO: These should be removed. GetSite does this already.
  pub admins: Vec<UserView>,   // hexbear
  pub sitemods: Vec<UserView>, // hexbear
}

#[derive(Serialize, Deserialize)]
pub struct GetRepliesResponse {
  pub replies: Vec<ReplyView>,
}

#[derive(Serialize, Deserialize)]
pub struct GetUserMentionsResponse {
  pub mentions: Vec<UserMentionView>,
}

#[derive(Serialize, Deserialize)]
pub struct MarkAllAsRead {
  pub auth: String,
}

#[derive(Serialize, Deserialize)]
pub struct AddAdmin {
  pub user_id: i32,
  pub added: bool,
  pub auth: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AddAdminResponse {
  pub admins: Vec<UserView>,
}

#[derive(Serialize, Deserialize)]
pub struct BanUser {
  pub user_id: i32,
  pub ban: bool,
  pub remove_data: Option<bool>,
  pub reason: Option<String>,
  pub expires: Option<i64>,
  pub auth: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct BanUserResponse {
  pub user: UserView,
  pub banned: bool,
}

#[derive(Serialize, Deserialize)]
pub struct GetReplies {
  pub sort: String,
  pub page: Option<i64>,
  pub limit: Option<i64>,
  pub unread_only: bool,
  pub auth: String,
}

#[derive(Serialize, Deserialize)]
pub struct GetUserMentions {
  pub sort: String,
  pub page: Option<i64>,
  pub limit: Option<i64>,
  pub unread_only: bool,
  pub auth: String,
}

#[derive(Serialize, Deserialize)]
pub struct MarkUserMentionAsRead {
  pub user_mention_id: i32,
  pub read: bool,
  pub auth: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct UserMentionResponse {
  pub mention: UserMentionView,
}

#[derive(Serialize, Deserialize)]
pub struct DeleteAccount {
  pub password: String,
  pub auth: String,
}

#[derive(Serialize, Deserialize)]
pub struct PasswordReset {
  pub email: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PasswordResetResponse {}

#[derive(Serialize, Deserialize)]
pub struct PasswordChange {
  pub token: String,
  pub password: String,
  pub password_verify: String,
}

#[derive(Serialize, Deserialize)]
pub struct CreatePrivateMessage {
  pub content: String,
  pub recipient_id: i32,
  pub auth: String,
}

#[derive(Serialize, Deserialize)]
pub struct EditPrivateMessage {
  pub edit_id: i32,
  pub content: String,
  pub auth: String,
}

#[derive(Serialize, Deserialize)]
pub struct DeletePrivateMessage {
  pub edit_id: i32,
  pub deleted: bool,
  pub auth: String,
}

#[derive(Serialize, Deserialize)]
pub struct MarkPrivateMessageAsRead {
  pub edit_id: i32,
  pub read: bool,
  pub auth: String,
}

#[derive(Serialize, Deserialize)]
pub struct GetPrivateMessages {
  pub unread_only: bool,
  pub page: Option<i64>,
  pub limit: Option<i64>,
  pub auth: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PrivateMessagesResponse {
  pub messages: Vec<PrivateMessageView>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PrivateMessageResponse {
  pub message: PrivateMessageView,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserJoin {
  pub auth: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct UserJoinResponse {
  pub user_id: i32,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct LeaveRooms {}

#[derive(Serialize, Deserialize, Clone)]
pub struct LeaveRoomsResponse {
  pub success: bool,
}

// Hexbear ------------------------------------------------------------

#[derive(Serialize, Deserialize)]
pub struct HCaptchaResponse {
  pub site_key: String,
  pub verify_url: String,
}

#[derive(Serialize, Deserialize)]
pub struct AddSitemod {
  pub user_id: i32,
  pub added: bool,
  pub auth: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AddSitemodResponse {
  pub sitemods: Vec<UserView>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct GetUserTag {
  pub user: i32,
  pub community: Option<i32>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SetUserTag {
  pub tag: String,
  pub value: Option<String>,
  pub auth: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct UserTagResponse {
  pub user: i32,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub community: Option<i32>,
  pub tags: UserTagsSchema,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct UserTagsSchema {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub pronouns: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub tendency: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub favorite_food: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub flair: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct RemoveUserContent {
  pub user_id: i32,
  pub time: Option<i32>,
  pub community_id: Option<i32>,
  pub reason: Option<String>,
  pub auth: String,
}

#[derive(Serialize, Deserialize)]
pub struct GetUnreadCount {
  pub auth: String,
}

#[derive(Serialize, Deserialize)]
pub struct GetUnreadCountResponse {
  pub unreads: i32,
}
