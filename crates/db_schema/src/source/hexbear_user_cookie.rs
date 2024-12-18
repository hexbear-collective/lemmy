use crate::newtypes::LocalUserId;
use crate::schema::user_cookie;
use crate::schema::user_cookie_local_users;
use chrono::{DateTime, Utc};
#[cfg(feature = "full")]
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use uuid::Uuid;

#[skip_serializing_none]
#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(table_name = user_cookie))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
pub struct HexbearUserCookie {
  pub cookie_uuid: Uuid,
}

#[skip_serializing_none]
#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(table_name = user_cookie_local_users))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
pub struct HexbearUserCookieLocalUsers {
  pub cookie_uuid: Uuid,
  pub local_user_id: LocalUserId,
}
