use crate::{
  newtypes::LocalUserId,
  schema::{user_cookie, user_cookie_local_users},
  source::hexbear_user_cookie::{HexbearUserCookie, HexbearUserCookieLocalUsers},
  utils::{get_conn, DbPool},
};
use diesel::prelude::*;
use diesel::{dsl::insert_into, result::Error, ExpressionMethods, OptionalExtension, QueryDsl};
use diesel_async::RunQueryDsl;
use uuid::Uuid;
impl HexbearUserCookie {
  pub async fn process_cookie(
    pool: &mut DbPool<'_>,
    local_user_id: LocalUserId,
    cookie_val: String,
  ) -> String {
    let hexbear_user_cookie = Self::get_by_user_id(pool, local_user_id).await.unwrap();
    if hexbear_user_cookie.is_none() {
      return Self::create(pool, local_user_id)
        .await
        .unwrap()
        .cookie_uuid
        .to_string();
    }
    let hexbear_user_cookie_uuid = hexbear_user_cookie.unwrap().cookie_uuid.to_string();

    if cookie_val == hexbear_user_cookie_uuid {
      return cookie_val.to_string();
    } else {
      let incoming_cookie_uuid = uuid::Uuid::parse_str(&cookie_val);
      if incoming_cookie_uuid.is_err() {
        return hexbear_user_cookie_uuid;
      }
      let found_other_cookie = Self::get_by_uuid(pool, incoming_cookie_uuid.unwrap())
        .await
        .unwrap();
      if (found_other_cookie.is_none()) {
        //fake cookie uuid? just return local users cookie to overwrite
        return hexbear_user_cookie_uuid;
      } else {
        //associate with found cookie
        let val = found_other_cookie.unwrap().cookie_uuid;
        Self::associate(pool, local_user_id, val).await;
        return val.to_string();
      }
    }
  }
  pub async fn create(
    pool: &mut DbPool<'_>,
    local_user_id: LocalUserId,
  ) -> Result<HexbearUserCookieLocalUsers, Error> {
    use crate::schema::user_cookie::dsl::user_cookie;
    use crate::schema::user_cookie_local_users::dsl::user_cookie_local_users;
    let conn = &mut get_conn(pool).await?;
    let cookie = insert_into(user_cookie)
      .default_values()
      .get_result::<Self>(conn)
      .await?;

    let form = HexbearUserCookieLocalUsers {
      cookie_uuid: cookie.cookie_uuid,
      local_user_id: local_user_id,
    };
    let user_association = insert_into(user_cookie_local_users)
      .values(&form)
      .get_result::<HexbearUserCookieLocalUsers>(conn)
      .await;

    return user_association;
  }

  pub async fn get_by_user_id(
    pool: &mut DbPool<'_>,
    local_user_id: LocalUserId,
  ) -> Result<Option<HexbearUserCookieLocalUsers>, Error> {
    let conn = &mut get_conn(pool).await?;

    user_cookie_local_users::table
      .filter(user_cookie_local_users::local_user_id.eq(local_user_id))
      .first(conn)
      .await
      .optional()
  }

  pub async fn get_by_uuid(
    pool: &mut DbPool<'_>,
    cookie_uuid: Uuid,
  ) -> Result<Option<HexbearUserCookie>, Error> {
    let conn = &mut get_conn(pool).await?;

    user_cookie::table
      .filter(user_cookie::cookie_uuid.eq(cookie_uuid))
      .first(conn)
      .await
      .optional()
  }

  pub async fn associate(
    pool: &mut DbPool<'_>,
    local_user_id: LocalUserId,
    other_cookie_uuid: Uuid,
  ) -> Result<HexbearUserCookieLocalUsers, Error> {
    use crate::schema::user_cookie::dsl::user_cookie;
    use crate::schema::user_cookie_local_users::dsl::user_cookie_local_users;
    let conn = &mut get_conn(pool).await?;

    let form = HexbearUserCookieLocalUsers {
      cookie_uuid: other_cookie_uuid,
      local_user_id: local_user_id,
    };
    let user_association = insert_into(user_cookie_local_users)
      .values(&form)
      .get_result::<HexbearUserCookieLocalUsers>(conn)
      .await;

    return user_association;
  }
}
