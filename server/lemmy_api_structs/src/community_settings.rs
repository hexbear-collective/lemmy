use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct GetCommunitySettings {
  pub community_id: i32,
  pub auth: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct GetCommunitySettingsResponse {
  pub read_only: bool,
  pub private: bool,
  pub post_links: bool,
  pub comment_images: i32,
  pub published: NaiveDateTime,
  pub allow_as_default: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EditCommunitySettings {
  pub community_id: i32,
  pub read_only: bool,
  pub private: bool,
  pub post_links: bool,
  pub comment_images: i32,
  pub allow_as_default: bool,
  pub auth: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct EditCommunitySettingsResponse {
  pub read_only: bool,
  pub private: bool,
  pub post_links: bool,
  pub comment_images: i32,
  pub published: NaiveDateTime,
  pub allow_as_default: bool,
}
