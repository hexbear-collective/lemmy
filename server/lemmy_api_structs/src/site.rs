use lemmy_db::{
  category::*,
  comment_view::*,
  community_view::*,
  moderator_views::*,
  post_view::*,
  site_view::*,
  user::*,
  user_view::*,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ListCategories {}

#[derive(Serialize, Deserialize)]
pub struct ListCategoriesResponse {
  pub categories: Vec<Category>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Search {
  pub q: String,
  pub type_: String,
  pub community_id: Option<i32>,
  pub sort: String,
  pub page: Option<i64>,
  pub limit: Option<i64>,
  pub auth: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SearchResponse {
  pub type_: String,
  pub comments: Vec<CommentView>,
  pub posts: Vec<PostView>,
  pub communities: Vec<CommunityView>,
  pub users: Vec<UserView>,
}

#[derive(Serialize, Deserialize)]
pub struct GetModlog {
  pub mod_user_id: Option<i32>,
  pub community_id: Option<i32>,
  pub page: Option<i64>,
  pub limit: Option<i64>,
  pub auth: Option<String>, // hexbear
}

#[derive(Serialize, Deserialize)]
pub struct GetModlogResponse {
  pub removed_posts: Vec<ModRemovePostView>,
  pub locked_posts: Vec<ModLockPostView>,
  pub stickied_posts: Vec<ModStickyPostView>,
  pub removed_comments: Vec<ModRemoveCommentView>,
  pub removed_communities: Vec<ModRemoveCommunityView>,
  pub banned_from_community: Vec<ModBanFromCommunityView>,
  pub banned: Vec<ModBanView>,
  pub added_to_community: Vec<ModAddCommunityView>,
  pub added: Vec<ModAddView>,
}

#[derive(Serialize, Deserialize)]
pub struct CreateSite {
  pub name: String,
  pub description: Option<String>,
  pub icon: Option<String>,
  pub banner: Option<String>,
  pub enable_downvotes: bool,
  pub open_registration: bool,
  pub enable_nsfw: bool,
  pub auth: String,
}

#[derive(Serialize, Deserialize)]
pub struct EditSite {
  pub name: String,
  pub description: Option<String>,
  pub icon: Option<String>,
  pub banner: Option<String>,
  pub enable_downvotes: bool,
  pub open_registration: bool,
  pub enable_nsfw: bool,
  pub auth: String,
  pub enable_create_communities: Option<bool>,
}

#[derive(Serialize, Deserialize)]
pub struct GetSite {
  pub auth: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SiteResponse {
  pub site: SiteView,
}

#[derive(Serialize, Deserialize)]
pub struct GetSiteResponse {
  pub site: Option<SiteView>,
  pub admins: Vec<UserView>,
  pub banned: Vec<UserView>,
  pub online: usize,
  pub version: String,
  pub my_user: Option<User_>,
  pub federated_instances: Vec<String>,
  pub sitemods: Vec<UserView>, // hexbear
}

#[derive(Serialize, Deserialize)]
pub struct TransferSite {
  pub user_id: i32,
  pub auth: String,
}

#[derive(Serialize, Deserialize)]
pub struct GetSiteConfig {
  pub auth: String,
}

#[derive(Serialize, Deserialize)]
pub struct GetSiteConfigResponse {
  pub config_hjson: String,
}

#[derive(Serialize, Deserialize)]
pub struct SaveSiteConfig {
  pub config_hjson: String,
  pub auth: String,
}

// Hexbear ----------------------------------------

#[derive(Serialize, Deserialize, Clone)]
pub struct CommunityModerators {
  pub community: CommunityView,
  pub moderators: Vec<i32>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct GetSiteModerators {
  pub page: Option<i64>,
  pub limit: Option<i64>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct GetSiteModeratorsResponse {
  pub communities: Vec<CommunityModerators>,
}
