use crate::Perform;
use actix_web::web::Data;
use lemmy_api_common::{
  context::LemmyContext,
  person::{HexbearGetRelatedUsers, HexbearGetRelatedUsersResponse},
  utils::{is_admin, local_user_view_from_jwt},
};
use lemmy_db_schema::{source::user_ban_id::UserBanId, utils::get_conn};
use lemmy_utils::error::LemmyError;

#[async_trait::async_trait(?Send)]
impl Perform for HexbearGetRelatedUsers {
  type Response = HexbearGetRelatedUsersResponse;

  async fn perform(&self, context: &Data<LemmyContext>) -> Result<Self::Response, LemmyError> {
    let data: &HexbearGetRelatedUsers = &self;

    let local_user_view = local_user_view_from_jwt(&data.auth, context).await?;

    // make sure they're an admin
    is_admin(&local_user_view)?;

    let userid = data.user_id;
    let userbanids =
      UserBanId::get_all_by_user(&mut get_conn(&mut context.pool()).await?, &userid).await;

    let mut found_users = vec![];
    match userbanids {
      Some(ubids) => {
        for ubid in ubids.iter() {
          let users =
            UserBanId::get_users_by_bid(&mut get_conn(&mut context.pool()).await?, ubid.bid)
              .await?;
          found_users.extend(users);
        }
        found_users.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        Ok(HexbearGetRelatedUsersResponse { users: found_users })
      }
      None => Ok(HexbearGetRelatedUsersResponse { users: vec![] }),
    }
  }
}
