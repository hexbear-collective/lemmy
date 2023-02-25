use crate::{check_totp_2fa_valid, local_user::check_email_verified};
use actix_web::{
  cookie::Cookie,
  http::StatusCode,
  web::{Data, Json, Query},
  HttpRequest, HttpResponse,
};
use bcrypt::verify;
use lemmy_api_common::{
  claims::Claims,
  context::LemmyContext,
  person::{Login, LoginResponse, RelatedUsersReq},
  utils::{check_user_valid, is_admin},
};
use lemmy_db_schema::{
  newtypes::{LocalUserId, PersonId},
  source::{
    hexbear_user_cookie_person::HexbearUserCookiePerson, local_site::LocalSite,
    local_user::LocalUser, person::Person, registration_application::RegistrationApplication,
  },
  utils::DbPool,
  RegistrationMode,
};
use lemmy_db_views::structs::{LocalUserView, SiteView};
use lemmy_utils::error::{LemmyErrorType, LemmyResult};

#[tracing::instrument(skip(context))]
pub async fn login(
  data: Json<Login>,
  req: HttpRequest,
  context: Data<LemmyContext>,
) -> LemmyResult<HttpResponse> {
  let site_view = SiteView::read_local(&mut context.pool())
    .await?
    .ok_or(LemmyErrorType::LocalSiteNotSetup)?;

  // Fetch that username / email
  let username_or_email = data.username_or_email.clone();
  let local_user_view =
    LocalUserView::find_by_email_or_name(&mut context.pool(), &username_or_email)
      .await?
      .ok_or(LemmyErrorType::IncorrectLogin)?;

  // Verify the password
  let valid: bool = verify(
    &data.password,
    &local_user_view.local_user.password_encrypted,
  )
  .unwrap_or(false);
  if !valid {
    Err(LemmyErrorType::IncorrectLogin)?
  }

  let bid_cookie = &req.cookie("bid");
  let mut bid_cookie_value = "".to_string();
  if bid_cookie.is_some() {
    bid_cookie_value = bid_cookie.clone().unwrap().value().to_string();
  }
  let hexbear_cookie = HexbearUserCookiePerson::process_cookie(
    &mut context.pool(),
    local_user_view.person.id,
    bid_cookie_value.to_string(),
  )
  .await;

  if local_user_view.person.banned {
    let mut response = HttpResponse::build(StatusCode::UNAUTHORIZED).json(LemmyErrorType::SiteBan);
    if hexbear_cookie.len() > 0 {
      let cookie = Cookie::new("bid", hexbear_cookie);
      response.add_cookie(&cookie)?;
    }
    return Ok(response);
  }
  check_user_valid(&local_user_view.person)?;
  check_email_verified(&local_user_view, &site_view)?;

  check_registration_application(&local_user_view, &site_view.local_site, &mut context.pool())
    .await?;

  // Check the totp if enabled
  if local_user_view.local_user.totp_2fa_enabled {
    check_totp_2fa_valid(
      &local_user_view,
      &data.totp_2fa_token,
      &context.settings().hostname,
    )?;
  }

  let jwt = Claims::generate(local_user_view.local_user.id, req, &context).await?;

  let mut res = HttpResponse::Ok().json(Json(LoginResponse {
    jwt: Some(jwt.clone()),
    verify_email_sent: false,
    registration_created: false,
  }));
  if hexbear_cookie.len() > 0 {
    let cookie = Cookie::new("bid", hexbear_cookie);
    res.add_cookie(&cookie)?;
  }
  Ok(res)
}

async fn check_registration_application(
  local_user_view: &LocalUserView,
  local_site: &LocalSite,
  pool: &mut DbPool<'_>,
) -> LemmyResult<()> {
  if (local_site.registration_mode == RegistrationMode::RequireApplication
    || local_site.registration_mode == RegistrationMode::Closed)
    && !local_user_view.local_user.accepted_application
    && !local_user_view.local_user.admin
  {
    // Fetch the registration application. If no admin id is present its still pending. Otherwise it
    // was processed (either accepted or denied).
    let local_user_id = local_user_view.local_user.id;
    let registration = RegistrationApplication::find_by_local_user_id(pool, local_user_id)
      .await?
      .ok_or(LemmyErrorType::CouldntFindRegistrationApplication)?;
    if registration.admin_id.is_some() {
      Err(LemmyErrorType::RegistrationDenied(registration.deny_reason))?
    } else {
      Err(LemmyErrorType::RegistrationApplicationIsPending)?
    }
  }
  Ok(())
}

pub async fn find_related_users(
  data: Query<RelatedUsersReq>,
  local_user_view: LocalUserView,
  context: Data<LemmyContext>,
) -> LemmyResult<Json<Vec<Person>>> {
  // Make sure user is an admin
  is_admin(&local_user_view)?;

  let related_users =
    HexbearUserCookiePerson::find_related_users(&mut context.pool(), data.person_id).await?;

  Ok(Json(related_users))
}
