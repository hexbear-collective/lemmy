use super::post_report_view::post_report_view::BoxedQuery;
use crate::{limit_and_offset, MaybeOptional};
use diesel::{pg::Pg, result::Error, *};
use serde::{Deserialize, Serialize};

table! {
    post_report_view (id) {
      id -> Uuid,
      time -> Timestamp,
      reason -> Nullable<Text>,
      resolved -> Bool,
      user_id -> Int4,
      post_id -> Int4,
      post_name -> Varchar,
      post_url -> Nullable<Text>,
      post_body -> Nullable<Text>,
      post_time -> Timestamp,
      community_id -> Int4,
    }
}
#[derive(
  Queryable, Identifiable, PartialEq, Debug, Serialize, Deserialize, QueryableByName, Clone,
)]
#[table_name = "post_report_view"]
pub struct PostReportView {
  pub id: uuid::Uuid,
  pub time: chrono::NaiveDateTime,
  pub reason: Option<String>,
  pub resolved: bool,
  pub user_id: i32,
  pub post_id: i32,
  pub post_name: String,
  pub post_url: Option<String>,
  pub post_body: Option<String>,
  pub post_time: chrono::NaiveDateTime,
  pub community_id: i32,
}

pub struct PostReportViewQueryBuilder<'a> {
  conn: &'a PgConnection,
  query: BoxedQuery<'a, Pg>,
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
