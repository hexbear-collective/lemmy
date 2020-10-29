use crate::{
  naive_now,
  schema::{user_tokens, user_tokens::dsl::*},
};
use chrono::Duration;
use diesel::{dsl::*, result::Error, *};
use serde::{Deserialize, Serialize};

#[derive(Clone, Queryable, Identifiable, PartialEq, Debug, Serialize, Deserialize)]
pub struct UserToken {
  pub id: uuid::Uuid,
  pub user_id: i32,
  pub token_hash: String,
  pub created_at: chrono::NaiveDateTime,
  pub expires_at: chrono::NaiveDateTime,
  pub renewed_at: chrono::NaiveDateTime,
  pub is_revoked: bool,
}

#[derive(Insertable, AsChangeset, Clone, Debug, Deserialize)]
#[table_name = "user_tokens"]
pub struct UserTokenForm {
  pub id: uuid::Uuid,
  pub user_id: i32,
  pub token_hash: String,
  pub expires_at: chrono::NaiveDateTime,
}

impl UserToken {
  pub fn create(conn: &PgConnection, form: &UserTokenForm) -> Result<Self, Error> {
    insert_into(user_tokens)
      .values(form)
      .get_result::<Self>(conn)
  }
  pub fn read(conn: &PgConnection, uuid: uuid::Uuid) -> Result<Self, Error> {
    user_tokens.find(uuid).first::<Self>(conn)
  }

  pub fn renew(conn: &PgConnection, uuid: uuid::Uuid, minutes: i64) -> Result<usize, Error> {
    diesel::update(user_tokens.find(uuid))
      .set((
        expires_at.eq(naive_now() + Duration::minutes(minutes)),
        renewed_at.eq(naive_now()),
      ))
      .execute(conn)
  }

  pub fn revoke(conn: &PgConnection, uuid: uuid::Uuid) -> Result<usize, Error> {
    diesel::update(user_tokens.find(uuid))
      .set(is_revoked.eq(true))
      .execute(conn)
  }

  pub fn revoke_all(conn: &PgConnection, other_user_id: i32) -> Result<usize, Error> {
    diesel::update(user_tokens.filter(user_id.eq(other_user_id)))
      .set(is_revoked.eq(true))
      .execute(conn)
  }
}
