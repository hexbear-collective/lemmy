use crate::{
  diesel::Connection,
  diesel_migrations::MigrationHarness,
  newtypes::DbUrl,
  CommentSortType,
  SortType,
};
use activitypub_federation::{fetch::object_id::ObjectId, traits::Object};
use chrono::{DateTime, Utc};
use deadpool::Runtime;
use diesel::{
  backend::Backend,
  deserialize::FromSql,
  helper_types::AsExprOf,
  pg::Pg,
  result::{ConnectionError, ConnectionResult, Error as DieselError, Error::QueryBuilderError},
  serialize::{Output, ToSql},
  sql_types::{Text, Timestamptz},
  IntoSql,
  PgConnection,
};
use diesel_async::{
  pg::AsyncPgConnection,
  pooled_connection::{
    deadpool::{Object as PooledConnection, Pool},
    AsyncDieselConnectionManager,
    ManagerConfig,
  },
};
use diesel_migrations::EmbeddedMigrations;
use futures_util::{future::BoxFuture, Future, FutureExt};
use lemmy_utils::{
  error::{LemmyError, LemmyErrorExt, LemmyErrorType},
  settings::SETTINGS,
};
use once_cell::sync::Lazy;
use regex::Regex;
use rustls::{
  client::{ServerCertVerified, ServerCertVerifier},
  ServerName,
};
use std::{
  ops::{Deref, DerefMut},
  sync::Arc,
  time::SystemTime,
};
use tracing::{error, info};
use url::Url;

const FETCH_LIMIT_DEFAULT: i64 = 10;
pub const FETCH_LIMIT_MAX: i64 = 50;
pub const SITEMAP_LIMIT: i64 = 50000;
pub const SITEMAP_DAYS: i64 = 31;
pub const RANK_DEFAULT: f64 = 0.0001;

pub type ActualDbPool = Pool<AsyncPgConnection>;

/// References a pool or connection. Functions must take `&mut DbPool<'_>` to allow implicit reborrowing.
///
/// https://github.com/rust-lang/rfcs/issues/1403
pub enum DbPool<'a> {
  Pool(&'a ActualDbPool),
  Conn(&'a mut AsyncPgConnection),
}

pub enum DbConn<'a> {
  Pool(PooledConnection<AsyncPgConnection>),
  Conn(&'a mut AsyncPgConnection),
}

pub async fn get_conn<'a, 'b: 'a>(pool: &'a mut DbPool<'b>) -> Result<DbConn<'a>, DieselError> {
  Ok(match pool {
    DbPool::Pool(pool) => DbConn::Pool(pool.get().await.map_err(|e| QueryBuilderError(e.into()))?),
    DbPool::Conn(conn) => DbConn::Conn(conn),
  })
}

impl<'a> Deref for DbConn<'a> {
  type Target = AsyncPgConnection;

  fn deref(&self) -> &Self::Target {
    match self {
      DbConn::Pool(conn) => conn.deref(),
      DbConn::Conn(conn) => conn.deref(),
    }
  }
}

impl<'a> DerefMut for DbConn<'a> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    match self {
      DbConn::Pool(conn) => conn.deref_mut(),
      DbConn::Conn(conn) => conn.deref_mut(),
    }
  }
}

// Allows functions that take `DbPool<'_>` to be called in a transaction by passing `&mut conn.into()`
impl<'a> From<&'a mut AsyncPgConnection> for DbPool<'a> {
  fn from(value: &'a mut AsyncPgConnection) -> Self {
    DbPool::Conn(value)
  }
}

impl<'a, 'b: 'a> From<&'a mut DbConn<'b>> for DbPool<'a> {
  fn from(value: &'a mut DbConn<'b>) -> Self {
    DbPool::Conn(value.deref_mut())
  }
}

impl<'a> From<&'a ActualDbPool> for DbPool<'a> {
  fn from(value: &'a ActualDbPool) -> Self {
    DbPool::Pool(value)
  }
}

/// Runs multiple async functions that take `&mut DbPool<'_>` as input and return `Result`. Only works when the  `futures` crate is listed in `Cargo.toml`.
///
/// `$pool` is the value given to each function.
///
/// A `Result` is returned (not in a `Future`, so don't use `.await`). The `Ok` variant contains a tuple with the values returned by the given functions.
///
/// The functions run concurrently if `$pool` has the `DbPool::Pool` variant.
#[macro_export]
macro_rules! try_join_with_pool {
  ($pool:ident => ($($func:expr),+)) => {{
    // Check type
    let _: &mut $crate::utils::DbPool<'_> = $pool;

    match $pool {
      // Run concurrently with `try_join`
      $crate::utils::DbPool::Pool(__pool) => ::futures::try_join!(
        $(async {
          let mut __dbpool = $crate::utils::DbPool::Pool(__pool);
          ($func)(&mut __dbpool).await
        }),+
      ),
      // Run sequentially
      $crate::utils::DbPool::Conn(__conn) => async {
        Ok(($({
          let mut __dbpool = $crate::utils::DbPool::Conn(__conn);
          // `?` prevents the error type from being inferred in an `async` block, so `match` is used instead
          match ($func)(&mut __dbpool).await {
            ::core::result::Result::Ok(__v) => __v,
            ::core::result::Result::Err(__v) => return ::core::result::Result::Err(__v),
          }
        }),+))
      }.await,
    }
  }};
}

pub fn fuzzy_search(q: &str) -> String {
  let replaced = q.replace('%', "\\%").replace('_', "\\_").replace(' ', "%");
  format!("%{replaced}%")
}

pub fn limit_and_offset(
  page: Option<i64>,
  limit: Option<i64>,
) -> Result<(i64, i64), diesel::result::Error> {
  let page = match page {
    Some(page) => {
      if page < 1 {
        return Err(QueryBuilderError("Page is < 1".into()));
      } else {
        page
      }
    }
    None => 1,
  };
  let limit = match limit {
    Some(limit) => {
      if !(1..=FETCH_LIMIT_MAX).contains(&limit) {
        return Err(QueryBuilderError(
          format!("Fetch limit is > {FETCH_LIMIT_MAX}").into(),
        ));
      } else {
        limit
      }
    }
    None => FETCH_LIMIT_DEFAULT,
  };
  let offset = limit * (page - 1);
  Ok((limit, offset))
}

pub fn limit_and_offset_unlimited(page: Option<i64>, limit: Option<i64>) -> (i64, i64) {
  let limit = limit.unwrap_or(FETCH_LIMIT_DEFAULT);
  let offset = limit * (page.unwrap_or(1) - 1);
  (limit, offset)
}

pub fn is_email_regex(test: &str) -> bool {
  EMAIL_REGEX.is_match(test)
}

pub fn diesel_option_overwrite(opt: Option<String>) -> Option<Option<String>> {
  match opt {
    // An empty string is an erase
    Some(unwrapped) => {
      if !unwrapped.eq("") {
        Some(Some(unwrapped))
      } else {
        Some(None)
      }
    }
    None => None,
  }
}

pub fn diesel_option_overwrite_to_url(
  opt: &Option<String>,
) -> Result<Option<Option<DbUrl>>, LemmyError> {
  match opt.as_ref().map(String::as_str) {
    // An empty string is an erase
    Some("") => Ok(Some(None)),
    Some(str_url) => Url::parse(str_url)
      .map(|u| Some(Some(u.into())))
      .with_lemmy_type(LemmyErrorType::InvalidUrl),
    None => Ok(None),
  }
}

pub fn diesel_option_overwrite_to_url_create(
  opt: &Option<String>,
) -> Result<Option<DbUrl>, LemmyError> {
  match opt.as_ref().map(String::as_str) {
    // An empty string is nothing
    Some("") => Ok(None),
    Some(str_url) => Url::parse(str_url)
      .map(|u| Some(u.into()))
      .with_lemmy_type(LemmyErrorType::InvalidUrl),
    None => Ok(None),
  }
}

fn establish_connection(config: &str) -> BoxFuture<ConnectionResult<AsyncPgConnection>> {
  let fut = async {
    // We first set up the way we want rustls to work.
    let mut rustls_config = rustls::ClientConfig::builder()
      .with_safe_defaults()
      .with_root_certificates(root_certs())
      .with_no_client_auth();
    rustls_config
      .dangerous()
      .set_certificate_verifier(Arc::new(danger::NoCertificateVerification {}));
    let tls = tokio_postgres_rustls::MakeRustlsConnect::new(rustls_config);
    let (client, conn) = tokio_postgres::connect(config, tls)
      .await
      .map_err(|e| ConnectionError::BadConnection(e.to_string()))?;
    tokio::spawn(async move {
      if let Err(e) = conn.await {
        error!("Database connection failed: {e}");
      }
    });
    AsyncPgConnection::try_from(client).await
  };
  fut.boxed()
}

fn root_certs() -> rustls::RootCertStore {
  let mut roots = rustls::RootCertStore::empty();
  let certs = rustls_native_certs::load_native_certs().expect("Certs not loadable!");
  let certs: Vec<_> = certs.into_iter().map(|cert| cert.0).collect();
  roots.add_parsable_certificates(&certs);
  roots
}

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

fn run_migrations(db_url: &str) {
  // Needs to be a sync connection
  let mut conn =
    PgConnection::establish(db_url).unwrap_or_else(|e| panic!("Error connecting to {db_url}: {e}"));
  info!("Running Database migrations (This may take a long time)...");
  let _ = &mut conn
    .run_pending_migrations(MIGRATIONS)
    .unwrap_or_else(|e| panic!("Couldn't run DB Migrations: {e}"));
  info!("Database migrations complete.");
}

pub async fn build_db_pool() -> Result<ActualDbPool, LemmyError> {
  let db_url = SETTINGS.get_database_url();
  // We only support TLS with sslmode=require currently
  let tls_enabled = db_url.contains("sslmode=require");
  let manager = if tls_enabled {
    // diesel-async does not support any TLS connections out of the box, so we need to manually
    // provide a setup function which handles creating the connection
    let mut config = ManagerConfig::default();
    config.custom_setup = Box::new(establish_connection);
    AsyncDieselConnectionManager::<AsyncPgConnection>::new_with_config(&db_url, config)
  } else {
    AsyncDieselConnectionManager::<AsyncPgConnection>::new(&db_url)
  };
  let pool = Pool::builder(manager)
    .max_size(SETTINGS.database.pool_size)
    .runtime(Runtime::Tokio1)
    .build()?;

  run_migrations(&db_url);

  Ok(pool)
}

pub async fn build_db_pool_for_tests() -> ActualDbPool {
  build_db_pool().await.expect("db pool missing")
}

pub fn naive_now() -> DateTime<Utc> {
  Utc::now()
}

pub fn post_to_comment_sort_type(sort: SortType) -> CommentSortType {
  match sort {
    SortType::Active | SortType::Hot | SortType::Scaled => CommentSortType::Hot,
    SortType::New | SortType::NewComments | SortType::MostComments => CommentSortType::New,
    SortType::Old => CommentSortType::Old,
    SortType::Controversial => CommentSortType::Controversial,
    SortType::TopHour
    | SortType::TopSixHour
    | SortType::TopTwelveHour
    | SortType::TopDay
    | SortType::TopAll
    | SortType::TopWeek
    | SortType::TopYear
    | SortType::TopMonth
    | SortType::TopThreeMonths
    | SortType::TopSixMonths
    | SortType::TopNineMonths => CommentSortType::Top,
  }
}

static EMAIL_REGEX: Lazy<Regex> = Lazy::new(|| {
  Regex::new(r"^[a-zA-Z0-9.!#$%&’*+/=?^_`{|}~-]+@[a-zA-Z0-9-]+(?:\.[a-zA-Z0-9-]+)*$")
    .expect("compile email regex")
});

pub mod functions {
  use diesel::sql_types::{BigInt, Text, Timestamptz};

  sql_function! {
    fn hot_rank(score: BigInt, time: Timestamptz) -> Double;
  }

  sql_function! {
    fn hot_rank_active(score: BigInt, time: Timestamptz, comment_time: Timestamptz) -> Double;
  }

  sql_function! {
    fn scaled_rank(score: BigInt, time: Timestamptz, users_active_month: BigInt) -> Double;
  }

  sql_function! {
    fn controversy_rank(upvotes: BigInt, downvotes: BigInt, score: BigInt) -> Double;
  }

  sql_function!(fn lower(x: Text) -> Text);

  // really this function is variadic, this just adds the two-argument version
  sql_function!(fn coalesce<T: diesel::sql_types::SqlType + diesel::sql_types::SingleValue>(x: diesel::sql_types::Nullable<T>, y: T) -> T);
}

pub const DELETED_REPLACEMENT_TEXT: &str = "*Permanently Deleted*";

impl ToSql<Text, Pg> for DbUrl {
  fn to_sql(&self, out: &mut Output<Pg>) -> diesel::serialize::Result {
    <std::string::String as ToSql<Text, Pg>>::to_sql(&self.0.to_string(), &mut out.reborrow())
  }
}

impl<DB: Backend> FromSql<Text, DB> for DbUrl
where
  String: FromSql<Text, DB>,
{
  fn from_sql(value: DB::RawValue<'_>) -> diesel::deserialize::Result<Self> {
    let str = String::from_sql(value)?;
    Ok(DbUrl(Box::new(Url::parse(&str)?)))
  }
}

impl<Kind> From<ObjectId<Kind>> for DbUrl
where
  Kind: Object + Send + 'static,
  for<'de2> <Kind as Object>::Kind: serde::Deserialize<'de2>,
{
  fn from(id: ObjectId<Kind>) -> Self {
    DbUrl(Box::new(id.into()))
  }
}

pub fn now() -> AsExprOf<diesel::dsl::now, diesel::sql_types::Timestamptz> {
  // https://github.com/diesel-rs/diesel/issues/1514
  diesel::dsl::now.into_sql::<Timestamptz>()
}

pub type ResultFuture<'a, T> = BoxFuture<'a, Result<T, DieselError>>;

pub trait ReadFn<'a, T, Args>: Fn(DbConn<'a>, Args) -> ResultFuture<'a, T> {}

impl<'a, T, Args, F: Fn(DbConn<'a>, Args) -> ResultFuture<'a, T>> ReadFn<'a, T, Args> for F {}

pub trait ListFn<'a, T, Args>: Fn(DbConn<'a>, Args) -> ResultFuture<'a, Vec<T>> {}

impl<'a, T, Args, F: Fn(DbConn<'a>, Args) -> ResultFuture<'a, Vec<T>>> ListFn<'a, T, Args> for F {}

/// Allows read and list functions to capture a shared closure that has an inferred return type, which is useful for join logic
pub struct Queries<RF, LF> {
  pub read_fn: RF,
  pub list_fn: LF,
}

// `()` is used to prevent type inference error
impl Queries<(), ()> {
  pub fn new<'a, RFut, LFut, RT, LT, RA, LA, RF2, LF2>(
    read_fn: RF2,
    list_fn: LF2,
  ) -> Queries<impl ReadFn<'a, RT, RA>, impl ListFn<'a, LT, LA>>
  where
    RFut: Future<Output = Result<RT, DieselError>> + Sized + Send + 'a,
    LFut: Future<Output = Result<Vec<LT>, DieselError>> + Sized + Send + 'a,
    RF2: Fn(DbConn<'a>, RA) -> RFut,
    LF2: Fn(DbConn<'a>, LA) -> LFut,
  {
    Queries {
      read_fn: move |conn, args| read_fn(conn, args).boxed(),
      list_fn: move |conn, args| list_fn(conn, args).boxed(),
    }
  }
}

impl<RF, LF> Queries<RF, LF> {
  pub async fn read<'a, T, Args>(
    self,
    pool: &'a mut DbPool<'_>,
    args: Args,
  ) -> Result<T, DieselError>
  where
    RF: ReadFn<'a, T, Args>,
  {
    let conn = get_conn(pool).await?;
    (self.read_fn)(conn, args).await
  }

  pub async fn list<'a, T, Args>(
    self,
    pool: &'a mut DbPool<'_>,
    args: Args,
  ) -> Result<Vec<T>, DieselError>
  where
    LF: ListFn<'a, T, Args>,
  {
    let conn = get_conn(pool).await?;
    (self.list_fn)(conn, args).await
  }
}

mod danger {
  pub struct NoCertificateVerification {}

  impl rustls::client::ServerCertVerifier for NoCertificateVerification {
    fn verify_server_cert(
      &self,
      _end_entity: &rustls::Certificate,
      _intermediates: &[rustls::Certificate],
      _server_name: &rustls::ServerName,
      _scts: &mut dyn Iterator<Item = &[u8]>,
      _ocsp: &[u8],
      _now: std::time::SystemTime,
    ) -> Result<rustls::client::ServerCertVerified, rustls::Error> {
      Ok(rustls::client::ServerCertVerified::assertion())
    }
  }
}

#[cfg(test)]
mod tests {
  #![allow(clippy::unwrap_used)]
  #![allow(clippy::indexing_slicing)]

  use super::{fuzzy_search, *};
  use crate::utils::is_email_regex;
  use pretty_assertions::assert_eq;

  #[test]
  fn test_fuzzy_search() {
    let test = "This %is% _a_ fuzzy search";
    assert_eq!(
      fuzzy_search(test),
      "%This%\\%is\\%%\\_a\\_%fuzzy%search%".to_string()
    );
  }

  #[test]
  fn test_email() {
    assert!(is_email_regex("gush@gmail.com"));
    assert!(!is_email_regex("nada_neutho"));
  }

  #[test]
  fn test_diesel_option_overwrite() {
    assert_eq!(diesel_option_overwrite(None), None);
    assert_eq!(diesel_option_overwrite(Some(String::new())), Some(None));
    assert_eq!(
      diesel_option_overwrite(Some("test".to_string())),
      Some(Some("test".to_string()))
    );
  }

  #[test]
  fn test_diesel_option_overwrite_to_url() {
    assert!(matches!(diesel_option_overwrite_to_url(&None), Ok(None)));
    assert!(matches!(
      diesel_option_overwrite_to_url(&Some(String::new())),
      Ok(Some(None))
    ));
    assert!(diesel_option_overwrite_to_url(&Some("invalid_url".to_string())).is_err());
    let example_url = "https://example.com";
    assert!(matches!(
      diesel_option_overwrite_to_url(&Some(example_url.to_string())),
      Ok(Some(Some(url))) if url == Url::parse(example_url).unwrap().into()
    ));
  }
}
