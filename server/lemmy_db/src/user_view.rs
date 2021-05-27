use crate::{fuzzy_search, limit_and_offset, MaybeOptional, SortType};

use diesel::{dsl::*, pg::Pg, query_builder::BoxedSelectStatement, result::Error, *};
use serde::{Deserialize, Serialize};

table! {
  hexbear.user_view (id) {
    id -> Int4,
    actor_id -> Text,
    name -> Varchar,
    preferred_username -> Nullable<Varchar>,
    avatar -> Nullable<Text>,
    banner -> Nullable<Text>,
    email -> Nullable<Text>,
    matrix_user_id -> Nullable<Text>,
    bio -> Nullable<Text>,
    local -> Bool,
    admin -> Bool,
    sitemod -> Bool,
    moderator -> Bool,
    banned -> Bool,
    show_avatars -> Bool,
    send_notifications_to_email -> Bool,
    published -> Timestamp,
    number_of_posts -> BigInt,
    post_score -> BigInt,
    number_of_comments -> BigInt,
    comment_score -> BigInt,
    has_2fa -> Bool,
    inbox_disabled -> Bool,
  }
}

#[derive(
  Queryable, Identifiable, PartialEq, Debug, Serialize, Deserialize, QueryableByName, Clone,
)]
#[table_name = "user_view"]
pub struct UserView {
  pub id: i32,
  pub actor_id: String,
  pub name: String,
  pub preferred_username: Option<String>,
  pub avatar: Option<String>,
  pub banner: Option<String>,
  pub email: Option<String>, // TODO this shouldn't be in this view
  pub matrix_user_id: Option<String>,
  pub bio: Option<String>,
  pub local: bool,
  pub admin: bool,
  pub sitemod: bool,
  pub moderator: bool,
  pub banned: bool,
  pub show_avatars: bool, // TODO this is a setting, probably doesn't need to be here
  pub send_notifications_to_email: bool, // TODO also never used
  pub published: chrono::NaiveDateTime,
  pub number_of_posts: i64,
  pub post_score: i64,
  pub number_of_comments: i64,
  pub comment_score: i64,
  pub has_2fa: bool,
  pub inbox_disabled: bool,
}

#[derive(
  Queryable, Identifiable, PartialEq, Debug, Serialize, Deserialize, QueryableByName, Clone,
)]
#[table_name = "user_view"]
pub struct UserViewSafe {
  pub id: i32,
  pub actor_id: String,
  pub name: String,
  pub preferred_username: Option<String>,
  pub avatar: Option<String>,
  pub banner: Option<String>,
  pub matrix_user_id: Option<String>,
  pub bio: Option<String>,
  pub local: bool,
  pub admin: bool,
  pub sitemod: bool,
  pub moderator: bool,
  pub banned: bool,
  pub published: chrono::NaiveDateTime,
  pub number_of_posts: i64,
  pub number_of_comments: i64,
}

pub struct UserQueryBuilder<'a> {
  conn: &'a PgConnection,
  query: BoxedSelectStatement<
    'a,
    (
      sql_types::Integer,
      sql_types::Text,
      sql_types::Text,
      sql_types::Nullable<sql_types::Text>,
      sql_types::Nullable<sql_types::Text>,
      sql_types::Nullable<sql_types::Text>,
      sql_types::Nullable<sql_types::Text>,
      sql_types::Nullable<sql_types::Text>,
      sql_types::Bool,
      sql_types::Bool,
      sql_types::Bool,
      sql_types::Bool,
      sql_types::Bool,
      sql_types::Timestamp,
      sql_types::BigInt,
      sql_types::BigInt,
    ),
    user_view::table,
    Pg,
  >,
  sort: &'a SortType,
  page: Option<i64>,
  limit: Option<i64>,
}

impl<'a> UserQueryBuilder<'a> {
  pub fn create(conn: &'a PgConnection) -> Self {
    use super::user_view::user_view::dsl::*;

    let query = user_view
      .select((
        id,
        actor_id,
        name,
        preferred_username,
        avatar,
        banner,
        matrix_user_id,
        bio,
        local,
        admin,
        sitemod,
        moderator,
        banned,
        published,
        number_of_posts,
        number_of_comments,
      ))
      .into_boxed();

    UserQueryBuilder {
      conn,
      query,
      sort: &SortType::Hot,
      page: None,
      limit: None,
    }
  }

  pub fn sort(mut self, sort: &'a SortType) -> Self {
    self.sort = sort;
    self
  }

  pub fn search_term<T: MaybeOptional<String>>(mut self, search_term: T) -> Self {
    use super::user_view::user_view::dsl::*;
    if let Some(search_term) = search_term.get_optional() {
      self.query = self.query.filter(name.ilike(fuzzy_search(&search_term)));
    }
    self
  }

  pub fn page<T: MaybeOptional<i64>>(mut self, page: T) -> Self {
    self.page = page.get_optional();
    self
  }

  pub fn limit<T: MaybeOptional<i64>>(mut self, limit: T) -> Self {
    self.limit = limit.get_optional();
    self
  }

  pub fn list(self) -> Result<Vec<UserViewSafe>, Error> {
    use super::user_view::user_view::dsl::*;
    let mut query = self.query;

    query = match self.sort {
      SortType::Hot => query
        .order_by(comment_score.desc())
        .then_order_by(published.desc()),
      SortType::Active => query
        .order_by(comment_score.desc())
        .then_order_by(published.desc()),
      SortType::New => query.order_by(published.desc()),
      SortType::TopAll => query.order_by(comment_score.desc()),
      SortType::TopYear => query
        .filter(published.gt(now - 1.years()))
        .order_by(comment_score.desc()),
      SortType::TopMonth => query
        .filter(published.gt(now - 1.months()))
        .order_by(comment_score.desc()),
      SortType::TopWeek => query
        .filter(published.gt(now - 1.weeks()))
        .order_by(comment_score.desc()),
      SortType::TopDay => query
        .filter(published.gt(now - 1.days()))
        .order_by(comment_score.desc()),
    };

    let (limit, offset) = limit_and_offset(self.page, self.limit);
    query = query.limit(limit).offset(offset);

    query.load::<UserViewSafe>(self.conn)
  }
}

impl UserView {
  pub fn read(conn: &PgConnection, from_user_id: i32) -> Result<Self, Error> {
    use super::user_view::user_view::dsl::*;
    user_view.find(from_user_id).first::<Self>(conn)
  }

  pub fn read_mult(conn: &PgConnection, from_user_ids: Vec<i32>) -> Result<Vec<Self>, Error> {
    use super::user_view::user_view::dsl::*;
    user_view.filter(id.eq(any(from_user_ids))).load(conn)
  }

  pub fn admins(conn: &PgConnection) -> Result<Vec<Self>, Error> {
    use super::user_view::user_view::dsl::*;
    use diesel::sql_types::{Nullable, Text};
    user_view
      // The select is necessary here to not get back emails
      .select((
        id,
        actor_id,
        name,
        preferred_username,
        avatar,
        banner,
        "".into_sql::<Nullable<Text>>(),
        matrix_user_id,
        bio,
        local,
        admin,
        sitemod,
        moderator,
        banned,
        show_avatars,
        send_notifications_to_email,
        published,
        number_of_posts,
        post_score,
        number_of_comments,
        comment_score,
        has_2fa,
        inbox_disabled,
      ))
      .filter(admin.eq(true))
      .order_by(published)
      .load::<Self>(conn)
  }

  pub fn sitemods(conn: &PgConnection) -> Result<Vec<Self>, Error> {
    use super::user_view::user_view::dsl::*;
    use diesel::sql_types::{Nullable, Text};
    user_view
      // The select is necessary here to not get back emails
      .select((
        id,
        actor_id,
        name,
        preferred_username,
        avatar,
        banner,
        "".into_sql::<Nullable<Text>>(),
        matrix_user_id,
        bio,
        local,
        admin,
        sitemod,
        moderator,
        banned,
        show_avatars,
        send_notifications_to_email,
        published,
        number_of_posts,
        post_score,
        number_of_comments,
        comment_score,
        has_2fa,
        inbox_disabled,
      ))
      .filter(sitemod.eq(true))
      .order_by(published)
      .load::<Self>(conn)
  }

  pub fn moderators(conn: &PgConnection) -> Result<Vec<Self>, Error> {
    use super::user_view::user_view::dsl::*;
    use diesel::sql_types::{Nullable, Text};
    user_view
      // The select is necessary here to not get back emails
      .select((
        id,
        actor_id,
        name,
        preferred_username,
        avatar,
        banner,
        "".into_sql::<Nullable<Text>>(),
        matrix_user_id,
        bio,
        local,
        admin,
        sitemod,
        moderator,
        banned,
        show_avatars,
        send_notifications_to_email,
        published,
        number_of_posts,
        post_score,
        number_of_comments,
        comment_score,
        has_2fa,
        inbox_disabled,
      ))
      .filter(moderator.eq(true))
      .order_by(published)
      .load::<Self>(conn)
  }

  pub fn banned(conn: &PgConnection) -> Result<Vec<Self>, Error> {
    use super::user_view::user_view::dsl::*;
    use diesel::sql_types::{Nullable, Text};
    user_view
      .select((
        id,
        actor_id,
        name,
        preferred_username,
        avatar,
        banner,
        "".into_sql::<Nullable<Text>>(),
        matrix_user_id,
        bio,
        local,
        admin,
        sitemod,
        moderator,
        banned,
        show_avatars,
        send_notifications_to_email,
        published,
        number_of_posts,
        post_score,
        number_of_comments,
        comment_score,
        has_2fa,
        inbox_disabled,
      ))
      .filter(banned.eq(true))
      .load::<Self>(conn)
  }

  pub fn get_user_secure(conn: &PgConnection, user_id: i32) -> Result<Self, Error> {
    use super::user_view::user_view::dsl::*;
    use diesel::sql_types::{Nullable, Text};
    user_view
      .select((
        id,
        actor_id,
        name,
        preferred_username,
        avatar,
        banner,
        "".into_sql::<Nullable<Text>>(),
        matrix_user_id,
        bio,
        local,
        admin,
        sitemod,
        moderator,
        banned,
        show_avatars,
        send_notifications_to_email,
        published,
        number_of_posts,
        post_score,
        number_of_comments,
        comment_score,
        has_2fa,
        inbox_disabled,
      ))
      .find(user_id)
      .first::<Self>(conn)
  }
}

impl UserViewSafe {
  pub fn read(conn: &PgConnection, from_user_id: i32) -> Result<Self, Error> {
    use super::user_view::user_view::dsl::*;
    user_view
      .select((
        id,
        actor_id,
        name,
        preferred_username,
        avatar,
        banner,
        matrix_user_id,
        bio,
        local,
        admin,
        sitemod,
        moderator,
        banned,
        published,
        number_of_posts,
        number_of_comments,
      ))
      .find(from_user_id)
      .first::<Self>(conn)
  }

  pub fn read_mult(conn: &PgConnection, from_user_ids: Vec<i32>) -> Result<Vec<Self>, Error> {
    use super::user_view::user_view::dsl::*;
    user_view
      .select((
        id,
        actor_id,
        name,
        preferred_username,
        avatar,
        banner,
        matrix_user_id,
        bio,
        local,
        admin,
        sitemod,
        moderator,
        banned,
        published,
        number_of_posts,
        number_of_comments,
      ))
      .filter(id.eq(any(from_user_ids)))
      .load(conn)
  }

  pub fn admins(conn: &PgConnection) -> Result<Vec<Self>, Error> {
    use super::user_view::user_view::dsl::*;
    user_view
      .select((
        id,
        actor_id,
        name,
        preferred_username,
        avatar,
        banner,
        matrix_user_id,
        bio,
        local,
        admin,
        sitemod,
        moderator,
        banned,
        published,
        number_of_posts,
        number_of_comments,
      ))
      .filter(admin.eq(true))
      .order_by(published)
      .load::<Self>(conn)
  }

  pub fn sitemods(conn: &PgConnection) -> Result<Vec<Self>, Error> {
    use super::user_view::user_view::dsl::*;
    user_view
      .select((
        id,
        actor_id,
        name,
        preferred_username,
        avatar,
        banner,
        matrix_user_id,
        bio,
        local,
        admin,
        sitemod,
        moderator,
        banned,
        published,
        number_of_posts,
        number_of_comments,
      ))
      .filter(sitemod.eq(true))
      .order_by(published)
      .load::<Self>(conn)
  }

  pub fn banned(conn: &PgConnection) -> Result<Vec<Self>, Error> {
    use super::user_view::user_view::dsl::*;
    user_view
      .select((
        id,
        actor_id,
        name,
        preferred_username,
        avatar,
        banner,
        matrix_user_id,
        bio,
        local,
        admin,
        sitemod,
        moderator,
        banned,
        published,
        number_of_posts,
        number_of_comments,
      ))
      .filter(banned.eq(true))
      .load::<Self>(conn)
  }
}
