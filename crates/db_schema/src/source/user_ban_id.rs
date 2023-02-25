use super::person::Person;
use crate::{
  schema::{ban_id, ban_id::dsl::*, person, user_ban_id, user_ban_id::dsl::*},
  utils::{get_conn, DbPool},
};
use diesel::{dsl::*, result::Error, *};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable))]
#[cfg_attr(feature = "full", diesel(table_name = ban_id))]
pub struct BanId {
  pub id: Uuid,
  pub created: chrono::NaiveDateTime,
  pub aliased_to: Option<Uuid>,
}

#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(Queryable))]
#[cfg_attr(feature = "full", diesel(table_name = user_ban_id))]
pub struct UserBanId {
  pub bid: Uuid,
  pub uid: i32,
}

#[derive(Clone, Default)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = user_ban_id))]
pub struct UserBanIdForm {
  pub bid: Uuid,
  pub uid: i32,
}

#[derive(Queryable)]
pub struct UserRelationResp {
  pub bid: Uuid,
  pub uid: i32,
  pub name: String,
  pub banned: bool,
}

impl BanId {
  pub async fn create(conn: &mut AsyncPgConnection) -> Result<Self, Error> {
    insert_into(ban_id)
      .default_values()
      .get_result::<Self>(conn)
      .await
  }

  pub async fn read(conn: &mut AsyncPgConnection, ban_id_val: Uuid) -> Result<Self, Error> {
    ban_id.find(ban_id_val).first::<Self>(conn).await
  }

  pub async fn read_opt(conn: &mut AsyncPgConnection, ban_id_val: Uuid) -> Option<Self> {
    let a = ban_id.find(ban_id_val).first::<Self>(conn).await;
    if a.is_err() {
      return None;
    }
    return Some(a.unwrap());
  }

  pub async fn update_alias(
    conn: &mut AsyncPgConnection,
    old_bid_val: Uuid,
    new_bid_val: Uuid,
  ) -> Result<Vec<Self>, Error> {
    update(ban_id.filter(ban_id::id.eq(old_bid_val).or(aliased_to.eq(old_bid_val))))
      .set(aliased_to.eq(new_bid_val))
      .get_results(conn)
      .await
  }
}

impl UserBanId {
  async fn simple_associate(
    conn: &mut AsyncPgConnection,
    ban_id_val: Uuid,
    user_id_val: i32,
  ) -> Result<Self, Error> {
    insert_into(user_ban_id)
      .values(UserBanIdForm {
        bid: ban_id_val,
        uid: user_id_val,
      })
      .get_result::<Self>(conn)
      .await
  }

  async fn overwriting_associate(
    conn: &mut AsyncPgConnection,
    old_bid_val: Uuid,
    new_bid_val: Uuid,
  ) -> Result<Self, Error> {
    BanId::update_alias(conn, old_bid_val, new_bid_val).await?;
    update(user_ban_id.filter(bid.eq(old_bid_val)))
      .set(bid.eq(new_bid_val))
      .get_result(conn)
      .await
  }

  pub async fn associate(pool: &DbPool, ban_id_val: Uuid, user_id_val: i32) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    match Self::get_by_user(conn, &user_id_val).await {
      //UserBanId found attached to user, which is not the same as the incoming one.
      Some(old_bid) if old_bid.bid != ban_id_val => {
        let incoming_bid = BanId::read(conn, ban_id_val).await?;
        //the incoming bid isn't aliased to the new one.
        if incoming_bid.aliased_to.is_none() || incoming_bid.aliased_to.unwrap() != old_bid.bid {
          return Self::overwriting_associate(conn, old_bid.bid, ban_id_val).await;
        }
        Ok(old_bid)
      }
      //UserBanId found, but it's the same as the incoming one.
      Some(k) => Ok(k),
      //There wasn't any UBID attached to the user. Associate and move on.
      None => {
        //Check for an alias
        let bid_read = BanId::read_opt(conn, ban_id_val).await;
        if let Some(BanId {
          aliased_to: Some(alias),
          ..
        }) = bid_read
        {
          Self::simple_associate(conn, alias, user_id_val).await
        } else {
          Self::simple_associate(conn, ban_id_val, user_id_val).await
        }
      }
    }
  }

  pub async fn create_then_associate(
    conn: &mut AsyncPgConnection,
    user_id_val: i32,
  ) -> Result<Self, Error> {
    let new_ban_id = BanId::create(conn).await?.id;
    Self::simple_associate(conn, new_ban_id, user_id_val).await
  }

  pub async fn get_by_user(conn: &mut AsyncPgConnection, user_id_val: &i32) -> Option<Self> {
    let a = user_ban_id
      .filter(uid.eq(user_id_val))
      .first::<Self>(conn)
      .await;
    if a.is_err() {
      return None;
    }
    return Some(a.unwrap());
  }

  pub async fn get_users_by_bid(
    conn: &mut AsyncPgConnection,
    ban_id_val: Uuid,
  ) -> Result<Vec<Person>, Error> {
    let uids = user_ban_id
      .filter(bid.eq(ban_id_val))
      .select(uid)
      .load::<i32>(conn)
      .await?;

    person::table
      .filter(person::id.eq_any(uids))
      .load::<Person>(conn)
      .await
  }
}
