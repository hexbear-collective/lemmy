use crate::ban_nonlocal_user_from_local_communities;
use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_common::{
  context::LemmyContext,
  person::{BanPerson, BanPersonResponse},
  send_activity::{ActivityChannel, SendActivityData},
  utils::{check_expire_time, is_admin, remove_or_restore_user_data},
};
use lemmy_db_schema::{
  source::{
    local_user::LocalUser,
    login_token::LoginToken,
    moderator::{ModBan, ModBanForm},
    person::{Person, PersonUpdateForm},
    user_ban_id::UserBanId,
  },
  traits::Crud,
  utils::get_conn,
};
use lemmy_db_views::structs::LocalUserView;
use lemmy_db_views_actor::structs::PersonView;
use lemmy_utils::{
  error::{LemmyErrorExt, LemmyErrorType, LemmyResult},
  utils::validation::is_valid_body_field,
};

#[tracing::instrument(skip(context))]
pub async fn ban_from_site(
  data: Json<BanPerson>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<BanPersonResponse>> {
  // Make sure user is an admin
  is_admin(&local_user_view)?;

  // Also make sure you're a higher admin than the target
  LocalUser::is_higher_admin_check(
    &mut context.pool(),
    local_user_view.person.id,
    vec![data.person_id],
  )
  .await?;

  if let Some(reason) = &data.reason {
    is_valid_body_field(reason, false)?;
  }

  let expires = check_expire_time(data.expires)?;

  let person = Person::update(
    &mut context.pool(),
    data.person_id,
    &PersonUpdateForm {
      banned: Some(data.ban),
      ban_expires: Some(expires),
      ..Default::default()
    },
  )
  .await
  .with_lemmy_type(LemmyErrorType::CouldntUpdateUser)?;

  // if its a local user, invalidate logins
  let local_user = LocalUserView::read_person(&mut context.pool(), person.id).await;
  if let Ok(local_user) = local_user {
    LoginToken::invalidate_all(&mut context.pool(), local_user.local_user.id).await?;
  }

  // Remove their data if that's desired
  if data.remove_or_restore_data.unwrap_or(false) {
    let removed = data.ban;
    remove_or_restore_user_data(
      local_user_view.person.id,
      person.id,
      removed,
      &data.reason,
      &context,
    )
    .await?;
  };

  // Mod tables
  let form = ModBanForm {
    mod_person_id: local_user_view.person.id,
    other_person_id: person.id,
    reason: data.reason.clone(),
    banned: Some(data.ban),
    expires,
  };

  ModBan::create(&mut context.pool(), &form).await?;

  let person_view = PersonView::read(&mut context.pool(), person.id).await?;

  ban_nonlocal_user_from_local_communities(
    &local_user_view,
    &person,
    data.ban,
    &data.reason,
    &data.remove_or_restore_data,
    &data.expires,
    &context,
  )
  .await?;

  ActivityChannel::submit_activity(
    SendActivityData::BanFromSite {
      moderator: local_user_view.person,
      banned_user: person_view.person.clone(),
      reason: data.reason.clone(),
      remove_or_restore_data: data.remove_or_restore_data,
      ban: data.ban,
      expires: data.expires,
    },
    &context,
  )?;

  Ok(Json(BanPersonResponse {
    person_view,
    banned: data.ban,
  }))
}
