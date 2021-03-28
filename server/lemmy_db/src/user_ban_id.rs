#[derive(Queryable)]
#[table_name = "hexbear.ban_id"]
pub struct BanId {
    pub id: i32,
    pub created: chrono::NaiveDateTime,
}

#[derive(Queryable)]
#[table_name = "hexbear.user_ban_id"]
pub struct UserBanId {
    pub bid: i32,
    pub uid: i32,
}