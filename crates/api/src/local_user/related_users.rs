use crate::Perform;
use actix_web::web::Data;
use lemmy_api_common::{
  context::LemmyContext,
  person::{HexbearGetRelatedUsers, HexbearGetRelatedUsersResponse},
  utils::{get_local_user_view_from_jwt, is_admin},
};
use lemmy_db_schema::{source::user_ban_id::UserBanId, utils::get_conn};
use lemmy_utils::error::LemmyError;

#[async_trait::async_trait(?Send)]
impl Perform for HexbearGetRelatedUsers {
  type Response = HexbearGetRelatedUsersResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<usize>,
  ) -> Result<Self::Response, LemmyError> {
    let data: &HexbearGetRelatedUsers = &self;

    let local_user_view =
      get_local_user_view_from_jwt(&data.auth, context.pool(), context.secret()).await?;

    // make sure they're an admin
    is_admin(&local_user_view)?;

    let userid = data.user_id;
    let conn = &mut get_conn(context.pool()).await?;
    let userbanids = UserBanId::get_all_by_user(conn, &userid).await;

    let mut foundUsers = vec![];
    match userbanids {
      Some(ubids) => {
        for ubid in ubids.iter() {
          let users = UserBanId::get_users_by_bid(conn, ubid.bid).await?;
          foundUsers.extend(users);
        }
        foundUsers.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        Ok(HexbearGetRelatedUsersResponse { users: foundUsers })
      }
      None => Ok(HexbearGetRelatedUsersResponse { users: vec![] }),
    }
  }
}
