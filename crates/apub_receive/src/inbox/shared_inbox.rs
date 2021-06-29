use crate::inbox::{
  assert_activity_not_local,
  is_activity_already_known,
  new_inbox_routing::{Activity, SharedInboxActivities},
};
use actix_web::{web, HttpRequest, HttpResponse};
use lemmy_apub::{
  check_is_apub_id_valid,
  extensions::signatures::verify_signature,
  fetcher::get_or_fetch_and_upsert_actor,
  insert_activity,
};
use lemmy_apub_lib::{ReceiveActivity, VerifyActivity};
use lemmy_utils::LemmyError;
use lemmy_websocket::LemmyContext;

pub async fn shared_inbox(
  request: HttpRequest,
  input: web::Json<Activity<SharedInboxActivities>>,
  context: web::Data<LemmyContext>,
) -> Result<HttpResponse, LemmyError> {
  let activity = input.into_inner();

  // Do nothing if we received the same activity before
  if is_activity_already_known(context.pool(), &activity.id_unchecked()).await? {
    return Ok(HttpResponse::Ok().finish());
  }
  assert_activity_not_local(&activity)?;
  check_is_apub_id_valid(&activity.actor, false)?;
  activity.inner.verify(&context).await?;

  let request_counter = &mut 0;
  let actor = get_or_fetch_and_upsert_actor(&activity.actor, &context, request_counter).await?;
  verify_signature(&request, actor.as_ref())?;

  // Log the activity, so we avoid receiving and parsing it twice. Note that this could still happen
  // if we receive the same activity twice in very quick succession.
  insert_activity(
    &activity.id_unchecked(),
    activity.clone(),
    false,
    true,
    context.pool(),
  )
  .await?;

  // TODO: pass the actor in somehow
  activity.inner.receive(&context, request_counter).await?;
  return Ok(HttpResponse::Ok().finish());
}
