use lemmy_db::{
  community_view::{CommunityFollowerView, CommunityModeratorView, CommunityView},
  user_view::UserViewSafe,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct GetCommunity {
  pub id: Option<i32>,
  pub name: Option<String>,
  pub auth: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct GetCommunityResponse {
  pub community: CommunityView,
  pub moderators: Vec<CommunityModeratorView>,
  pub online: usize,
  pub admins: Vec<UserViewSafe>,   // hexbear
  pub sitemods: Vec<UserViewSafe>, // hexbear
}

#[derive(Serialize, Deserialize)]
pub struct CreateCommunity {
  pub name: String,
  pub title: String,
  pub description: Option<String>,
  pub icon: Option<String>,
  pub banner: Option<String>,
  pub category_id: i32,
  pub nsfw: bool,
  pub auth: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CommunityResponse {
  pub community: CommunityView,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ListCommunities {
  pub sort: String,
  pub page: Option<i64>,
  pub limit: Option<i64>,
  pub auth: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ListCommunitiesResponse {
  pub communities: Vec<CommunityView>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct BanFromCommunity {
  pub community_id: i32,
  pub user_id: i32,
  pub ban: bool,
  pub remove_data: Option<bool>,
  pub reason: Option<String>,
  pub expires: Option<i64>,
  pub auth: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct BanFromCommunityResponse {
  pub user: UserViewSafe,
  pub banned: bool,
}

#[derive(Serialize, Deserialize)]
pub struct AddModToCommunity {
  pub community_id: i32,
  pub user_id: i32,
  pub added: bool,
  pub auth: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AddModToCommunityResponse {
  pub moderators: Vec<CommunityModeratorView>,
}

#[derive(Serialize, Deserialize)]
pub struct EditCommunity {
  pub edit_id: i32,
  pub title: String,
  pub description: Option<String>,
  pub icon: Option<String>,
  pub banner: Option<String>,
  pub category_id: i32,
  pub nsfw: bool,
  pub auth: String,
}

#[derive(Serialize, Deserialize)]
pub struct DeleteCommunity {
  pub edit_id: i32,
  pub deleted: bool,
  pub auth: String,
}

#[derive(Serialize, Deserialize)]
pub struct RemoveCommunity {
  pub edit_id: i32,
  pub removed: bool,
  pub reason: Option<String>,
  pub expires: Option<i64>,
  pub auth: String,
}

#[derive(Serialize, Deserialize)]
pub struct FollowCommunity {
  pub community_id: i32,
  pub follow: bool,
  pub auth: String,
}

#[derive(Serialize, Deserialize)]
pub struct GetFollowedCommunities {
  pub auth: String,
}

#[derive(Serialize, Deserialize)]
pub struct GetFollowedCommunitiesResponse {
  pub communities: Vec<CommunityFollowerView>,
}

#[derive(Serialize, Deserialize)]
pub struct TransferCommunity {
  pub community_id: i32,
  pub user_id: i32,
  pub auth: String,
}

#[derive(Serialize, Deserialize)]
pub struct CommunityJoinRoom {
  pub community_id: i32,
}

#[derive(Serialize, Deserialize)]
pub struct CommunityJoinRoomResponse {
  pub community_id: i32,
}