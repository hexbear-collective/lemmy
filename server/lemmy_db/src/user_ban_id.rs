use diesel::{dsl::*, result::Error, *};
use crate::schema::{
    {ban_id, ban_id::dsl::*},
    {user_ban_id, user_ban_id::dsl::*}
};
use uuid::Uuid;

#[derive(Queryable, Insertable)]
#[table_name = "ban_id"]
pub struct BanId {
    pub id: Uuid,
    pub created: chrono::NaiveDateTime,
    pub aliased_to: Option<Uuid>,
}

#[derive(Queryable, Insertable)]
#[table_name = "user_ban_id"]
pub struct UserBanId {
    pub bid: Uuid,
    pub uid: i32,
}

impl BanId {
    pub fn create(conn: &PgConnection) -> Result<Self, Error> {
        insert_into(ban_id).default_values().get_result::<Self>(conn)
    }

    pub fn read(conn: &PgConnection, ban_id_val: Uuid) -> Result<Self, Error> {
        ban_id.find(ban_id_val).first::<Self>(conn)
    }

    pub fn read_opt(conn: &PgConnection, ban_id_val: Uuid) -> Result<Option<Self>, Error> {
        ban_id.find(ban_id_val).first::<Self>(conn).optional()
    }

    pub fn update_alias(conn: &PgConnection, old_bid_val: Uuid, new_bid_val: Uuid) -> Result<Vec<Self>, Error> {
        update(ban_id.filter(id.eq(old_bid_val).or(aliased_to.eq(old_bid_val)))).set(aliased_to.eq(new_bid_val)).get_results(conn)
    }
}

impl UserBanId {
    fn simple_associate(conn: &PgConnection, ban_id_val: Uuid, user_id_val: i32) -> Result<Self, Error> {
        insert_into(user_ban_id)
            .values(UserBanId { bid: ban_id_val, uid: user_id_val })
            .get_result::<Self>(conn)
    }

    fn overwriting_associate(conn: &PgConnection, old_bid_val: Uuid, new_bid_val: Uuid) -> Result<Self, Error> {
        BanId::update_alias(conn, old_bid_val, new_bid_val)?;
        update(user_ban_id.filter(bid.eq(old_bid_val))).set(bid.eq(new_bid_val)).get_result(conn)
    }

    pub fn associate(conn: &PgConnection, ban_id_val: Uuid, user_id_val: i32) -> Result<Self, Error> {
        return match Self::get_by_user(conn, user_id_val) {
            //UserBanId found attached to user, which is not the same as the incoming one.
            Ok(Some(old_bid)) if old_bid.bid != ban_id_val => {
                let incoming_bid = BanId::read(conn, ban_id_val)?;
                //the incoming bid isn't aliased to the new one.
                if incoming_bid.aliased_to.is_none() || incoming_bid.aliased_to.unwrap() != old_bid.bid {
                    return Self::overwriting_associate(conn, old_bid.bid, ban_id_val);
                }
                Ok(old_bid)
            },
            //UserBanId found, but it's the same as the incoming one.
            Ok(Some(k)) => Ok(k),
            //There wasn't any UBID attached to the user. Associate and move on.
            Ok(None) => {
                //Check for an alias
                let bid_read = BanId::read_opt(conn, ban_id_val)?;
                if bid_read.is_some() && bid_read.as_ref().unwrap().aliased_to.is_some() {
                    Self::simple_associate(conn, bid_read.unwrap().aliased_to.unwrap(), user_id_val)
                } else {
                    Self::simple_associate(conn, ban_id_val, user_id_val)
                }
            },
            //Breaking error, bubble it up.
            Err(e) => Err(e),
        }
    }

    pub fn create_then_associate(conn: &PgConnection, user_id_val: i32) -> Result<Self, Error> {
        Self::simple_associate(conn, BanId::create(conn)?.id, user_id_val)
    }

    pub fn get_by_user(conn: &PgConnection, user_id_val: i32) -> Result<Option<Self>, Error> {
        user_ban_id.filter(uid.eq(user_id_val)).first::<Self>(conn).optional()
    }
}