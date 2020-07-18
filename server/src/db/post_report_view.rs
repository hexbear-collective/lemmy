use super::post_report_view::post_report_view::BoxedQuery;
use crate::db::{limit_and_offset, MaybeOptional};
use diesel::{pg::Pg, result::Error, *};
use serde::{Deserialize, Serialize};

table! {
    post_report_view (id) {
      id -> Int4,
      post_id -> Int4,
      user_id -> Int4,
      reason -> Nullable<Text>,
      time -> Timestamp,
      resolved -> Bool,
      community_id -> Int4,
      title -> Varchar,
      post_content_body -> Nullable<Text>,
      post_content_url -> Nullable<Text>,
      username -> Varchar,
      banned -> Bool,
      community_name -> Varchar,
    }
}
#[derive(
  Queryable, Identifiable, PartialEq, Debug, Serialize, Deserialize, QueryableByName, Clone,
)]
#[table_name = "post_report_view"]
pub struct PostReportView {
  pub id: i32,
  pub post_id: i32,
  pub user_id: i32,
  pub reason: Option<String>,
  pub time: chrono::NaiveDateTime,
  pub resolved: bool,
  pub community_id: i32,
  pub title: String,
  pub post_content_body: Option<String>,
  pub post_content_url: Option<String>,
  pub username: String,
  pub banned: bool,
  pub community_name: String,
}

pub struct PostReportViewQueryBuilder<'a> {
  conn: &'a PgConnection,
  query: BoxedQuery<'a, Pg>,
  for_creator_id: Option<i32>,
  for_community_id: Option<i32>,
  page: Option<i64>,
  limit: Option<i64>,
  resolved: Option<bool>,
}

impl<'a> PostReportViewQueryBuilder<'a> {
  pub fn create(conn: &'a PgConnection) -> Self {
    use super::post_report_view::post_report_view::dsl::*;

    let query = post_report_view.into_boxed();

    PostReportViewQueryBuilder {
      conn,
      query,
      for_creator_id: None,
      for_community_id: None,
      page: None,
      limit: None,
      resolved: Some(false),
    }
  }

  pub fn community_id<T: MaybeOptional<i32>>(mut self, community_id: T) -> Self {
    self.for_community_id = community_id.get_optional();
    self
  }

  pub fn creator_id<T: MaybeOptional<i32>>(mut self, creator_id: T) -> Self {
    self.for_creator_id = creator_id.get_optional();
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

  pub fn resolved<T: MaybeOptional<bool>>(mut self, resolved: T) -> Self {
    self.resolved = resolved.get_optional();
    self
  }

  pub fn list(self) -> Result<Vec<PostReportView>, Error> {
    use super::post_report_view::post_report_view::dsl::*;

    let mut query = self.query;

    if let Some(creator_id) = self.for_creator_id {
      query = query.filter(user_id.eq(creator_id));
    }

    if let Some(comm_id) = self.for_community_id {
      query = query.filter(community_id.eq(comm_id));
    }

    if let Some(resolved_flag) = self.resolved {
      query = query.filter(resolved.eq(resolved_flag));
    }

    let (limit, offset) = limit_and_offset(self.page, self.limit);

    query
      .limit(limit)
      .offset(offset)
      .load::<PostReportView>(self.conn)
  }
}
