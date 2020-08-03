use crate::schema::{user_tag, user_tag::dsl::*};
use diesel::{dsl::*, result::Error, *};

#[derive(Clone, Queryable, Identifiable, Insertable, PartialEq, AsChangeset, Debug)]
#[primary_key(user_id)]
#[table_name = "user_tag"]
pub struct UserTag {
  pub user_id: i32,
  pub tags: serde_json::Value,
}

impl UserTag {
  pub fn create(conn: &PgConnection, user: i32, t: &serde_json::Value) -> Result<Self, Error> {
    insert_into(user_tag)
      .values(UserTag {
        user_id: user,
        tags: t.to_owned(),
      })
      .get_result::<Self>(conn)
  }

  pub fn read(conn: &PgConnection, user: i32) -> Result<Self, Error> {
    user_tag.filter(user_id.eq(user)).first::<Self>(conn)
  }

  pub fn update(conn: &PgConnection, user: i32, t: &serde_json::Value) -> Result<Self, Error> {
    diesel::update(user_tag.find(user))
      .set(tags.eq(t))
      .get_result::<Self>(conn)
  }

  pub fn delete(conn: &PgConnection, user: i32) -> Result<usize, Error> {
    diesel::delete(user_tag.find(user)).execute(conn)
  }

  pub fn set_key(
    conn: &PgConnection,
    user: i32,
    tag_key: String,
    tag_value: Option<String>,
  ) -> Result<Self, Error> {
    let mut json = json!({});

    if let Ok(usertag) = user_tag.filter(user_id.eq(user)).first::<Self>(conn) {
      json = usertag.tags;
    }

    match tag_value {
      Some(value) => {
        json[tag_key] = serde_json::Value::String(value);
      }
      None => {
        json[tag_key].take();
      }
    }

    insert_into(user_tag)
      .values(UserTag {
        user_id: user,
        tags: json.to_owned(),
      })
      .on_conflict(user_id)
      .do_update()
      .set(tags.eq(json))
      .get_result::<Self>(conn)
  }
}
