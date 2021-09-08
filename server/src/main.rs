#[macro_use]
extern crate diesel_migrations;
#[macro_use]
pub extern crate lazy_static;

use actix::prelude::*;
use actix_cors::Cors;
use actix_web::{*, body::Body, dev::{Service, ServiceRequest, ServiceResponse}, http::{
    header::{CACHE_CONTROL, CONTENT_TYPE},
    HeaderValue,
  }, web::Data};
use diesel::{
  r2d2::{ConnectionManager, Pool},
  PgConnection,
};
use futures::Future;
use lemmy_db::get_database_url_from_env;
use lemmy_rate_limit::{rate_limiter::RateLimiter, RateLimit};
use lemmy_server::{
  apub::activity_queue::create_activity_queue, blocking, routes::*, twofactor::CodeCacheHandler,
  websocket::chat_server::ChatServer, LemmyContext,
};
use lemmy_utils::{
  settings::Settings, LemmyError, CACHE_CONTROL_APPLICATION_REGEX, CACHE_CONTROL_IMAGE_REGEX,
};
use reqwest::Client;
use std::sync::{Arc, Mutex};

lazy_static! {
  // static ref CACHE_CONTROL_VALUE: String = format!("public, max-age={}", 365 * 24 * 60 * 60);
  // Test out 1 hour here, this is breaking some things
  static ref CACHE_CONTROL_IMAGE_VALUE: String = format!("public, max-age={}", 12 * 60 * 60);
  static ref CACHE_CONTROL_APPLICATION_VALUE: String = format!("public, no-cache, max-age={}", 0);
}

embed_migrations!();

#[actix_rt::main]
async fn main() -> Result<(), LemmyError> {
  env_logger::init();
  let settings = Settings::get();

  // Set up the r2d2 connection pool
  let db_url = match get_database_url_from_env() {
    Ok(url) => url,
    Err(_) => settings.get_database_url(),
  };
  let manager = ConnectionManager::<PgConnection>::new(&db_url);
  let pool = Pool::builder()
    .max_size(settings.database.pool_size)
    .build(manager)
    .unwrap_or_else(|_| panic!("Error connecting to {}", db_url));

  // Run the migrations from code
  blocking(&pool, move |conn| {
    embedded_migrations::run(conn)?;
    // uncomment if there's an advanced migration we need to run
    // run_advanced_migrations(conn)?;
    Ok(()) as Result<(), LemmyError>
  })
  .await??;

  // Set up the rate limiter
  let rate_limiter = RateLimit {
    rate_limiter: Arc::new(Mutex::new(RateLimiter::default())),
  };

  println!(
    "Starting http server at {}:{}",
    settings.bind, settings.port
  );

  let activity_queue = create_activity_queue();
  let cache_handler = Arc::new(CodeCacheHandler::new());
  let chat_server = ChatServer::startup(
    pool.clone(),
    rate_limiter.clone(),
    Client::default(),
    activity_queue.clone(),
    cache_handler.clone(),
  )
  .start();

  // Create Http server with websocket support
  HttpServer::new(move || {
    let context = LemmyContext::create(
      pool.clone(),
      chat_server.to_owned(),
      Client::default(),
      activity_queue.to_owned(),
      cache_handler.clone(),
    );

    let cors = Cors::default()
      .allow_any_origin()
      .supports_credentials()
      .allowed_methods(vec!["GET", "POST", "PUT", "OPTIONS"])
      .allow_any_header()
      .max_age(3600);

    let rate_limiter = rate_limiter.clone();
    App::new()
      .wrap_fn(add_cache_headers)
      .wrap(cors)
      .wrap(middleware::Logger::default())
      .app_data(Data::new(context))
      // The routes
      .configure(|cfg| api::config(cfg, &rate_limiter))
      .configure(federation::config)
      .configure(feeds::config)
      .configure(|cfg| images::config(cfg, &rate_limiter))
      .configure(nodeinfo::config)
      .configure(webfinger::config)
  })
  .bind((settings.bind, settings.port))?
  .run()
  .await?;

  Ok(())
}

fn add_cache_headers<S>(
  req: ServiceRequest,
  srv: &S,
) -> impl Future<Output = Result<ServiceResponse, Error>>
where
  S: Service<ServiceRequest, Response = ServiceResponse<Body>, Error = Error>
{
  let fut = srv.call(req);
  async move {
    let mut res = fut.await?;
    if let Some(content_type) = res.headers().get(CONTENT_TYPE) {
      if CACHE_CONTROL_IMAGE_REGEX.is_match(content_type.to_str().unwrap()) {
        let header_val = HeaderValue::from_static(&CACHE_CONTROL_IMAGE_VALUE);
        res.headers_mut().insert(CACHE_CONTROL, header_val);
      } else if CACHE_CONTROL_APPLICATION_REGEX.is_match(content_type.to_str().unwrap()) {
        let header_val = HeaderValue::from_static(&CACHE_CONTROL_APPLICATION_VALUE);
        res.headers_mut().insert(CACHE_CONTROL, header_val);
      }
    }
    Ok(res)
  }
}
