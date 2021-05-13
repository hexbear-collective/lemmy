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
  pub users: Vec<UserViewSafe>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetModlog {
  pub mod_user_id: Option<i32>,
  pub other_user_id: Option<i32>,
  pub community_id: Option<i32>,
  pub page: Option<i64>,
  pub limit: Option<i64>,
  pub action_filter: Option<u16>,   //9 bits for each type of mod action
  pub auth: Option<String>, // hexbear
}

//exists in upstream, but is different from hexbear ----------
#[derive(Serialize)]
pub struct GetModlogResponse {
  pub log: Vec<ModlogAction>
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]    //enum type is inline with the internal data
pub enum ModlogAction {
  RemovePost(ModRemovePostView),
  LockPost(ModLockPostView),
  StickyPost(ModStickyPostView),
  RemoveComment(ModRemoveCommentView),
  RemoveCommunity(ModRemoveCommunityView),
  BanFromCommunity(ModBanFromCommunityView),
  BanFromSite(ModBanView),
  AddModToCommunity(ModAddCommunityView),
  AddMod(ModAddView),
}
//-------------------------------------------------------------

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
  pub autosubscribe_comms: Vec<i32>,
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
  pub version: String,
  pub online: usize,
  pub my_user: Option<User_>,
  pub admins: Vec<UserViewSafe>,
  pub sitemods: Vec<UserViewSafe>, // hexbear
  pub federated_instances: Vec<String>,
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
