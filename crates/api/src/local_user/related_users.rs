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
    let userbanid = UserBanId::get_by_user(conn, &userid).await;

    match userbanid {
      Some(ubid) => {
        let users = UserBanId::get_users_by_bid(conn, ubid.bid).await?;
        Ok(HexbearGetRelatedUsersResponse { users })
      }
      None => Ok(HexbearGetRelatedUsersResponse { users: vec![] }),
    }
  }
}
