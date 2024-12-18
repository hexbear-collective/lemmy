use crate::{
  newtypes::PersonId,
  schema::{person, user_cookie_person},
  source::{hexbear_user_cookie_person::HexbearUserCookiePerson, person::Person},
  utils::{get_conn, DbPool},
};
use diesel::{dsl::insert_into, result::Error, ExpressionMethods, OptionalExtension, QueryDsl};
use diesel_async::RunQueryDsl;
use uuid::Uuid;
impl HexbearUserCookiePerson {
  pub async fn process_cookie(
    pool: &mut DbPool<'_>,
    person_id: PersonId,
    cookie_val: String,
  ) -> String {
    let incoming_cookie_uuid = uuid::Uuid::parse_str(&cookie_val);
    let hexbear_user_cookies = Self::get_by_person_id(pool, person_id).await.unwrap();
    let found_other_cookie = Self::get_by_uuid_and_person(
      pool,
      incoming_cookie_uuid.unwrap_or(Uuid::default()),
      person_id,
    )
    .await
    .unwrap();
    if hexbear_user_cookies.is_empty() && found_other_cookie.is_none() {
      return Self::create(pool, person_id)
        .await
        .unwrap()
        .cookie_uuid
        .to_string();
    }
    //If cookie matches any db-loaded cookies, its valid so return it.
    for uuid in &hexbear_user_cookies {
      if cookie_val == uuid.cookie_uuid.to_string() {
        return cookie_val.to_string();
      }
    }
    if found_other_cookie.is_none() {
      return hexbear_user_cookies
        .into_iter()
        .nth(0)
        .unwrap()
        .cookie_uuid
        .to_string();
    } else {
      //associate with found cookie
      let val = found_other_cookie.unwrap().cookie_uuid;
      Self::associate(pool, person_id, val).await;
      return val.to_string();
    }
  }
  async fn create(
    pool: &mut DbPool<'_>,
    person_id: PersonId,
  ) -> Result<HexbearUserCookiePerson, Error> {
    use crate::schema::user_cookie_person::dsl::user_cookie_person;
    let conn = &mut get_conn(pool).await?;
    let form = HexbearUserCookiePerson {
      cookie_uuid: Uuid::new_v4(),
      person_id: person_id,
    };
    let user_association = insert_into(user_cookie_person)
      .values(&form)
      .get_result::<HexbearUserCookiePerson>(conn)
      .await;

    return user_association;
  }

  async fn get_by_person_id(
    pool: &mut DbPool<'_>,
    person_id: PersonId,
  ) -> Result<Vec<HexbearUserCookiePerson>, Error> {
    let conn = &mut get_conn(pool).await?;

    user_cookie_person::table
      .filter(user_cookie_person::person_id.eq(person_id))
      .get_results::<HexbearUserCookiePerson>(conn)
      .await
  }

  async fn get_users_by_uuid(
    pool: &mut DbPool<'_>,
    uuid: Uuid,
  ) -> Result<Vec<HexbearUserCookiePerson>, Error> {
    let conn = &mut get_conn(pool).await?;

    user_cookie_person::table
      .filter(user_cookie_person::cookie_uuid.eq(uuid))
      .get_results::<HexbearUserCookiePerson>(conn)
      .await
  }

  async fn get_by_uuid_and_person(
    pool: &mut DbPool<'_>,
    cookie_uuid: Uuid,
    person_id: PersonId,
  ) -> Result<Option<HexbearUserCookiePerson>, Error> {
    let conn = &mut get_conn(pool).await?;

    user_cookie_person::table
      .filter(user_cookie_person::cookie_uuid.eq(cookie_uuid))
      .filter(user_cookie_person::person_id.eq(person_id))
      .first(conn)
      .await
      .optional()
  }

  async fn get_related_usernames(
    pool: &mut DbPool<'_>,
    cookie_uuids: &Vec<Uuid>,
  ) -> Result<Vec<Person>, Error> {
    let conn = &mut get_conn(pool).await?;

    let user_cookies = user_cookie_person::table
      .filter(user_cookie_person::cookie_uuid.eq_any(cookie_uuids))
      .get_results::<HexbearUserCookiePerson>(conn)
      .await?;
    let user_ids = user_cookies
      .into_iter()
      .map(|x| x.person_id)
      .collect::<Vec<PersonId>>();
    return person::table
      .filter(person::id.eq_any(user_ids))
      .order(person::display_name)
      .select(person::all_columns)
      .get_results::<Person>(conn)
      .await;
  }

  async fn associate(
    pool: &mut DbPool<'_>,
    person_id: PersonId,
    other_cookie_uuid: Uuid,
  ) -> Result<HexbearUserCookiePerson, Error> {
    use crate::schema::user_cookie_person::dsl::user_cookie_person;
    let conn = &mut get_conn(pool).await?;

    let form = HexbearUserCookiePerson {
      cookie_uuid: other_cookie_uuid,
      person_id: person_id,
    };
    let user_association = insert_into(user_cookie_person)
      .values(&form)
      .get_result::<HexbearUserCookiePerson>(conn)
      .await;

    return user_association;
  }

  pub async fn find_related_users(
    pool: &mut DbPool<'_>,
    person_id: PersonId,
  ) -> Result<Vec<Person>, Error> {
    let conn = &mut get_conn(pool).await?;

    let found_users = Self::get_by_person_id(pool, person_id).await?;
    let mut related_user_uuids: &mut Vec<Uuid> = &mut Vec::new();
    for user in &found_users {
      Self::find_related_users_recursive(pool, user, &mut related_user_uuids).await?;
      related_user_uuids.push(user.cookie_uuid);
    }
    return Self::get_related_usernames(pool, related_user_uuids).await;
  }

  async fn find_related_users_recursive(
    pool: &mut DbPool<'_>,
    person_cookie: &HexbearUserCookiePerson,
    related_users: &mut Vec<Uuid>,
  ) -> Result<bool, Error> {
    let conn = &mut get_conn(pool).await?;
    let found_users = Self::get_users_by_uuid(pool, person_cookie.cookie_uuid).await?;
    for user in &found_users {
      if related_users.contains(&user.cookie_uuid) {
        continue;
      }
      Self::find_related_users_recursive(pool, user, related_users);
      related_users.push(user.cookie_uuid);
    }
    return Ok(true);
  }
}
