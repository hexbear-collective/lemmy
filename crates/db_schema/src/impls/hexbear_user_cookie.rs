use crate::{
  newtypes::LocalUserId, source::hexbear_user_cookie::{HexbearUserCookie, HexbearUserCookieLocalUsers}, utils::{get_conn, DbPool}
};
use diesel::prelude::*;
use diesel::{dsl::insert_into, result::Error, ExpressionMethods, OptionalExtension, QueryDsl};
use diesel_async::RunQueryDsl;
impl HexbearUserCookie {
  pub async fn process_cookie(pool: &mut DbPool<'_>, local_user_id: LocalUserId, cookie_val: String) -> Result<String, Error> {
    if cookie_val.len() == 0{
        return Self::create(pool,local_user_id);
    }
    return cookie;
  }
  pub async fn create(pool: &mut DbPool<'_>,local_user_id: LocalUserId) -> Result<HexbearUserCookie, Error> {
    use crate::schema::user_cookie::dsl::user_cookie;
    use crate::schema::user_cookie_local_users::dsl::user_cookie_local_users;
    let conn = &mut get_conn(pool).await?;
    let cookie = insert_into(user_cookie)
      .default_values()
      .get_result::<Self>(conn)
      .await;
    
    let form = HexbearUserCookieLocalUsers {
        cookie_uuid: cookie.unwrap().
    }
    let user_association = insert_into(user_cookie_local_users)
    .values({cookie_uuid})

    return cookie;
  }
}
