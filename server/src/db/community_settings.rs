use crate::{
  db::{Bannable, Crud, Followable, Joinable},
  schema::{community, community_settings},
};
use diesel::{dsl::*, result::Error, *};
use serde::{Deserialize, Serialize};

#[derive(Clone, Queryable, Identifiable, PartialEq, Debug, Serialize, Deserialize)]
#[table_name = "community_settings"]
pub struct CommunitySettings {
  pub community_id: i32,
  pub read_only: bool,
  pub hidden: bool,
  pub post_links: bool,
  pub post_images: bool,
  pub comment_images: i32,
  pub published: Option<chrono::NaiveDateTime>,
}

pub struct CommunitySettingsForm {
  pub community_id: i32,
  pub read_only: bool,
  pub hidden: bool,
  pub post_links: bool,
  pub post_images: bool,
  pub comment_images: i32,
  pub published: Option<chrono::NaiveDateTime>,
}

impl CommunitySettings {
  pub fn read_from_actor_id(conn: &PgConnection, community_id: &str) -> Result<Self, Error> {
    use crate::schema::community_settings::dsl::*;
    community_settings
      .filter(actor_id.eq(community_id))
      .first::<Self>(conn)
  }

  // idk maybe???
  pub fn list_local(conn: &PgConnection) -> Result<Vec<Self>, Error> {
    use crate::schema::community::dsl::*;
    community_settings
      .filter(local.eq(true))
      .load::<Community>(conn)
  }
}

impl Crud<CommunitySettingsForm> for CommunitySettings {
  fn read(conn: &PgConnection, community_id: i32) -> Result<Self, Error> {
    use crate::schema::community_settings::dsl::*;
    community_settings.find(community_id).first::<Self>(conn)
  }

  fn delete(conn: &PgConnection, community_id: i32) -> Result<usize, Error> {
    use crate::schema::community_settings::dsl::*;
    diesel::delete(community_settings.find(community_id)).execute(conn)
  }

  fn create(
    conn: &PgConnection,
    new_community_settings: &CommunitySettingsForm,
  ) -> Result<Self, Error> {
    use crate::schema::community_settings::dsl::*;
    insert_into(community_settings)
      .values(new_community_settings)
      .get_result::<Self>(conn)
  }

  fn update(
    conn: &PgConnection,
    community_id: i32,
    new_community_settings: &CommunitySettingsForm,
  ) -> Result<Self, Error> {
    use crate::schema::community_settings::dsl::*;
    diesel::update(community_settings.find(community_id))
      .set(new_community_settings)
      .get_result::<Self>(conn)
  }
}
