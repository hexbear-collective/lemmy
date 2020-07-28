use crate::schema::{user_tag, user_tag::dsl::*};
use diesel::{dsl::*, result::Error, *};

#[derive(Clone, Queryable, Identifiable, Insertable, PartialEq, Debug)]
#[primary_key(user_id, tag_name)]
#[table_name = "user_tag"]
pub struct UserTag {
  pub user_id: i32,
  pub tag_name: String,
  pub tag_value: String,
}

impl UserTag {
  pub fn create(conn: &PgConnection, user: i32, tag: String, value: String) -> Result<Self, Error> {
    insert_into(user_tag)
      .values(UserTag {
	user_id: user,
	tag_name: tag,
	tag_value: value,
      })
      .get_result::<Self>(conn)
  }

  pub fn read(conn: &PgConnection, user: i32) -> Result<Vec<Self>, Error> {
    user_tag.filter(user_id.eq(user)).get_results::<Self>(conn)
  } 
  
  pub fn read_tag(conn: &PgConnection, user: i32, tag: String) -> Result<Self, Error> {
    user_tag.find((user, tag)).first::<Self>(conn)
  }

  pub fn update(conn: &PgConnection, user: i32, tag: String, value: String) -> Result<Self, Error> {
    diesel::update(user_tag.find((user, tag)))
      .set(tag_value.eq(value))
      .get_result::<Self>(conn)
  }

  pub fn delete(conn: &PgConnection, user: i32, tag: String) -> Result<usize, Error> {
    diesel::delete(user_tag.find((user, tag))).execute(conn)
  }
}
