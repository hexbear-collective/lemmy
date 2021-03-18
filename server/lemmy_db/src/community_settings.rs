use crate::{schema::community_settings, Crud};
use diesel::{dsl::*, result::Error, *};
use serde::{Deserialize, Serialize};

#[derive(Clone, Queryable, Identifiable, PartialEq, Debug, Serialize, Deserialize)]
#[table_name = "community_settings"]
pub struct CommunitySettings {
  pub id: i32,
  pub read_only: bool,
  pub private: bool,
  pub post_links: bool,
  pub comment_images: i32,
  pub published: chrono::NaiveDateTime,
  pub allow_as_default: bool,
  pub hide_from_all: bool,
}

#[derive(Insertable, AsChangeset, Clone, Serialize, Deserialize, Debug)]
#[table_name = "community_settings"]
pub struct CommunitySettingsForm {
  pub id: i32,
  pub read_only: bool,
  pub private: bool,
  pub post_links: bool,
  pub comment_images: i32,
  pub allow_as_default: bool,
  pub hide_from_all: bool,
}

impl CommunitySettings {
  pub fn read_from_community_id(conn: &PgConnection, community_id_: i32) -> Result<Self, Error> {
    use crate::schema::community_settings::dsl::*;
    community_settings.find(community_id_).first::<Self>(conn)
  }

  pub fn list_allowed_as_default(conn: &PgConnection) -> Result<Vec<Self>, Error> {
    use crate::schema::community_settings::dsl::*;
    community_settings
      .filter(allow_as_default.eq(true))
      .load::<CommunitySettings>(conn)
  }
}

impl Crud<CommunitySettingsForm> for CommunitySettings {
  fn read(conn: &PgConnection, _id: i32) -> Result<Self, Error> {
    use crate::schema::community_settings::dsl::*;
    community_settings.find(_id).first::<Self>(conn)
  }

  fn delete(conn: &PgConnection, community_id_: i32) -> Result<usize, Error> {
    use crate::schema::community_settings::dsl::*;
    diesel::delete(community_settings.find(community_id_)).execute(conn)
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
    community_id_: i32,
    new_community_settings: &CommunitySettingsForm,
  ) -> Result<Self, Error> {
    use crate::schema::community_settings::dsl::*;
    diesel::update(community_settings.find(community_id_))
      .set(new_community_settings)
      .get_result::<Self>(conn)
  }
}
