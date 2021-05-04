use crate::{
  is_email_regex,
  naive_now,
  schema::{user_, user_::dsl::*},
  Crud,
};
use bcrypt::{hash, DEFAULT_COST};
use diesel::{dsl::*, result::Error, sql_types::Integer, *};
use serde::{Deserialize, Serialize};

// TextOrNullableText is a marker trait for Text or Nullable<Text>
// if/when it is eventuly removed form diesel this needs to be changed
sql_function!(fn lower<TT: TextOrNullableText>(x: TT) -> sql_types::Text);

#[derive(Clone, Queryable, Identifiable, PartialEq, Debug, Serialize, Deserialize)]
#[table_name = "user_"]
pub struct User_ {
  pub id: i32,
  pub name: String,
  pub preferred_username: Option<String>,
  pub password_encrypted: String,
  pub email: Option<String>,
  pub avatar: Option<String>,
  pub admin: bool,
  pub banned: bool,
  pub published: chrono::NaiveDateTime,
  pub updated: Option<chrono::NaiveDateTime>,
  pub show_nsfw: bool,
  pub theme: String,
  pub default_sort_type: i16,
  pub default_listing_type: i16,
  pub lang: String,
  pub show_avatars: bool,
  pub send_notifications_to_email: bool,
  pub matrix_user_id: Option<String>,
  pub actor_id: String,
  pub bio: Option<String>,
  pub local: bool,
  pub private_key: Option<String>,
  pub public_key: Option<String>,
  pub last_refreshed_at: chrono::NaiveDateTime,
  pub sitemod: bool,
  pub banner: Option<String>,
  pub has_2fa: bool,
  pub inbox_disabled: bool,
}

#[derive(Insertable, AsChangeset, Clone, Debug)]
#[table_name = "user_"]
pub struct UserForm {
  pub name: String,
  pub preferred_username: Option<String>,
  pub password_encrypted: String,
  pub admin: bool,
  pub banned: bool,
  pub email: Option<Option<String>>,
  pub avatar: Option<Option<String>>,
  pub updated: Option<chrono::NaiveDateTime>,
  pub show_nsfw: bool,
  pub theme: String,
  pub default_sort_type: i16,
  pub default_listing_type: i16,
  pub lang: String,
  pub show_avatars: bool,
  pub send_notifications_to_email: bool,
  pub matrix_user_id: Option<String>,
  pub actor_id: Option<String>,
  pub bio: Option<String>,
  pub local: bool,
  pub private_key: Option<String>,
  pub public_key: Option<String>,
  pub last_refreshed_at: Option<chrono::NaiveDateTime>,
  pub banner: Option<Option<String>>,
  pub has_2fa: bool,
  pub inbox_disabled: bool,
}

#[derive(QueryableByName)]
pub struct UserUnreadCount {
  #[sql_type = "Integer"]
  pub unreads: i32,
}

impl Crud<UserForm> for User_ {
  fn read(conn: &PgConnection, user_id: i32) -> Result<Self, Error> {
    user_.find(user_id).first::<Self>(conn)
  }
  fn delete(conn: &PgConnection, user_id: i32) -> Result<usize, Error> {
    diesel::delete(user_.find(user_id)).execute(conn)
  }
  fn create(conn: &PgConnection, form: &UserForm) -> Result<Self, Error> {
    insert_into(user_).values(form).get_result::<Self>(conn)
  }
  fn update(conn: &PgConnection, user_id: i32, form: &UserForm) -> Result<Self, Error> {
    diesel::update(user_.find(user_id))
      .set(form)
      .get_result::<Self>(conn)
  }
}

impl User_ {
  pub fn register(conn: &PgConnection, form: &UserForm) -> Result<Self, Error> {
    let mut edited_user = form.clone();
    let password_hash =
      hash(&form.password_encrypted, DEFAULT_COST).expect("Couldn't hash password");
    edited_user.password_encrypted = password_hash;

    Self::create(&conn, &edited_user)
  }

  // TODO do more individual updates like these
  pub fn update_password(
    conn: &PgConnection,
    user_id: i32,
    new_password: &str,
  ) -> Result<Self, Error> {
    let password_hash = hash(new_password, DEFAULT_COST).expect("Couldn't hash password");

    diesel::update(user_.find(user_id))
      .set((
        password_encrypted.eq(password_hash),
        updated.eq(naive_now()),
      ))
      .get_result::<Self>(conn)
  }

  pub fn update_username(conn: &PgConnection, user_id: i32, new_uname: String, new_actor: String) -> Result<Self, Error> {
    diesel::update(user_.find(user_id))
        .set((name.eq(new_uname.clone()), actor_id.eq(new_actor)))
        .get_result::<Self>(conn)
  }

  pub fn read_from_name(conn: &PgConnection, from_user_name: &str) -> Result<Self, Error> {
    user_
      .filter(lower(name).eq(from_user_name.to_lowercase()))
      .first::<Self>(conn)
  }

  pub fn add_admin(conn: &PgConnection, user_id: i32, added: bool) -> Result<Self, Error> {
    diesel::update(user_.find(user_id))
      .set(admin.eq(added))
      .get_result::<Self>(conn)
  }

  pub fn add_sitemod(conn: &PgConnection, user_id: i32, added: bool) -> Result<Self, Error> {
    diesel::update(user_.find(user_id))
      .set(sitemod.eq(added))
      .get_result::<Self>(conn)
  }

  pub fn ban_user(conn: &PgConnection, user_id: i32, ban: bool) -> Result<Self, Error> {
    diesel::update(user_.find(user_id))
      .set(banned.eq(ban))
      .get_result::<Self>(conn)
  }

  pub fn read_from_actor_id(conn: &PgConnection, object_id: &str) -> Result<Self, Error> {
    use crate::schema::user_::dsl::*;
    user_.filter(actor_id.eq(object_id)).first::<Self>(conn)
  }

  pub fn find_by_email_or_username(
    conn: &PgConnection,
    username_or_email: &str,
  ) -> Result<Self, Error> {
    if is_email_regex(username_or_email) {
      Self::find_by_email(conn, username_or_email)
    } else {
      Self::find_by_username(conn, username_or_email)
    }
  }

  pub fn find_by_username(conn: &PgConnection, username: &str) -> Result<User_, Error> {
    user_.filter(name.ilike(username)).first::<User_>(conn)
  }

  pub fn find_by_username_mult(conn: &PgConnection, username: &str) -> Result<Vec<User_>, Error> {
    user_.filter(name.ilike(username)).load::<User_>(conn)
  }

  pub fn find_by_email(conn: &PgConnection, from_email: &str) -> Result<Self, Error> {
    user_
      .filter(lower(email).eq(from_email.to_lowercase()))
      .first::<User_>(conn)
  }

  pub fn get_profile_url(&self, hostname: &str) -> String {
    format!("https://{}/u/{}", hostname, self.name)
  }

  pub fn upsert(conn: &PgConnection, user_form: &UserForm) -> Result<User_, Error> {
    insert_into(user_)
      .values(user_form)
      .on_conflict(actor_id)
      .do_update()
      .set(user_form)
      .get_result::<Self>(conn)
  }

  pub fn get_unread_notifs(conn: &PgConnection, user_id: i32) -> Result<UserUnreadCount, Error> {
    sql_query(
      "with post_replies as (
        select count(c.id)
        from post p
        join comment c on c.post_id = p.id
        where p.creator_id = $1 and c.creator_id <> p.creator_id and c.read is false
        and c.deleted is false and c.removed is false
        and p.deleted is false and p.removed is false
        and c.parent_id is null
      ),
      comment_replies as (
        select count(c2.id)
        from comment c
        join comment c2 on c.id = c2.parent_id
        where c.creator_id = $1 and c.creator_id <> c2.creator_id and c2.read is false
        and c.deleted is false and c.removed is false
        and c2.deleted is false and c2.removed is false
      )
      select
        coalesce((select count(*) from public.private_message where read is false and recipient_id = $1) +
        (select count(*) from public.user_mention where read is false and recipient_id = $1) +
        (select count from post_replies) + (select count from comment_replies), 0)::int as \"unreads\"")
    .bind::<Integer,_>(user_id)
    .get_result::<UserUnreadCount>(conn)
  }
}

#[cfg(test)]
mod tests {
  use crate::{tests::establish_unpooled_connection, user::*, ListingType, SortType};

  #[test]
  fn test_crud() {
    let conn = establish_unpooled_connection();

    let new_user = UserForm {
      name: "thommy".into(),
      preferred_username: None,
      password_encrypted: "nope".into(),
      email: None,
      matrix_user_id: None,
      avatar: None,
      banner: None,
      admin: false,
      banned: false,
      updated: None,
      show_nsfw: false,
      theme: "darkly".into(),
      default_sort_type: SortType::Hot as i16,
      default_listing_type: ListingType::Subscribed as i16,
      lang: "browser".into(),
      show_avatars: true,
      send_notifications_to_email: false,
      has_2fa: false,
      actor_id: None,
      bio: None,
      local: true,
      private_key: None,
      public_key: None,
      last_refreshed_at: None,
      inbox_disabled: false,
    };

    let inserted_user = User_::create(&conn, &new_user).unwrap();

    let expected_user = User_ {
      id: inserted_user.id,
      name: "thommy".into(),
      preferred_username: None,
      password_encrypted: "nope".into(),
      email: None,
      matrix_user_id: None,
      avatar: None,
      banner: None,
      admin: false,
      sitemod: false,
      banned: false,
      published: inserted_user.published,
      updated: None,
      show_nsfw: false,
      theme: "darkly".into(),
      default_sort_type: SortType::Hot as i16,
      default_listing_type: ListingType::Subscribed as i16,
      lang: "browser".into(),
      show_avatars: true,
      send_notifications_to_email: false,
      has_2fa: false,
      actor_id: inserted_user.actor_id.to_owned(),
      bio: None,
      local: true,
      private_key: None,
      public_key: None,
      last_refreshed_at: inserted_user.published,
      inbox_disabled: false,
    };

    let read_user = User_::read(&conn, inserted_user.id).unwrap();
    let updated_user = User_::update(&conn, inserted_user.id, &new_user).unwrap();
    let num_deleted = User_::delete(&conn, inserted_user.id).unwrap();

    assert_eq!(expected_user, read_user);
    assert_eq!(expected_user, inserted_user);
    assert_eq!(expected_user, updated_user);
    assert_eq!(1, num_deleted);
  }

  // #[test]
  // fn test_user_methods() {
  //   use crate::{community::*, Joinable};
  //   let conn = establish_unpooled_connection();
  //
  //   let new_creator_user = UserForm {
  //     name: "creator".into(),
  //     preferred_username: None,
  //     password_encrypted: "creator".into(),
  //     admin: false,
  //     email: None,
  //     matrix_user_id: None,
  //     avatar: None,
  //     banner: None,
  //     banned: false,
  //     updated: None,
  //     show_nsfw: false,
  //     theme: "darkly".into(),
  //     default_sort_type: SortType::Hot as i16,
  //     default_listing_type: ListingType::Subscribed as i16,
  //     lang: "browser".into(),
  //     show_avatars: true,
  //     send_notifications_to_email: false,
  //     actor_id: "http://fake.com".into(),
  //     bio: None,
  //     local: true,
  //     private_key: None,
  //     public_key: None,
  //     last_refreshed_at: None,
  //   };
  //
  //   let new_mod_user = UserForm {
  //     name: "moderator".into(),
  //     preferred_username: None,
  //     password_encrypted: "mod".into(),
  //     admin: false,
  //     email: None,
  //     matrix_user_id: None,
  //     avatar: None,
  //     banner: None,
  //     banned: false,
  //     updated: None,
  //     show_nsfw: false,
  //     theme: "darkly".into(),
  //     default_sort_type: SortType::Hot as i16,
  //     default_listing_type: ListingType::Subscribed as i16,
  //     lang: "browser".into(),
  //     show_avatars: true,
  //     send_notifications_to_email: false,
  //     actor_id: "http://fake.com".into(),
  //     bio: None,
  //     local: true,
  //     private_key: None,
  //     public_key: None,
  //     last_refreshed_at: None,
  //   };
  //
  //   let new_notmod_user = UserForm {
  //     name: "not_moderator".into(),
  //     preferred_username: None,
  //     password_encrypted: "nope".into(),
  //     admin: false,
  //     email: None,
  //     matrix_user_id: None,
  //     avatar: None,
  //     banner: None,
  //     banned: false,
  //     updated: None,
  //     show_nsfw: false,
  //     theme: "darkly".into(),
  //     default_sort_type: SortType::Hot as i16,
  //     default_listing_type: ListingType::Subscribed as i16,
  //     lang: "browser".into(),
  //     show_avatars: true,
  //     send_notifications_to_email: false,
  //     actor_id: "http://fake.com".into(),
  //     bio: None,
  //     local: true,
  //     private_key: None,
  //     public_key: None,
  //     last_refreshed_at: None,
  //   };
  //
  //   let inserted_creator_user = User_::create(&conn, &new_creator_user).unwrap();
  //   let inserted_mod_user = User_::create(&conn, &new_mod_user).unwrap();
  //   let inserted_notmod_user = User_::create(&conn, &new_notmod_user).unwrap();
  //
  //   let new_community = CommunityForm {
  //     name: "mod_community".to_string(),
  //     title: "nada".to_owned(),
  //     description: None,
  //     category_id: 1,
  //     creator_id: inserted_creator_user.id,
  //     removed: None,
  //     deleted: None,
  //     updated: None,
  //     nsfw: false,
  //     actor_id: "http://fake.com".into(),
  //     local: true,
  //     private_key: None,
  //     public_key: None,
  //     last_refreshed_at: None,
  //     published: None,
  //     banner: None,
  //     icon: None,
  //   };
  //
  //   let inserted_community = Community::create(&conn, &new_community).unwrap();
  //   let new_moderator = CommunityModeratorForm {
  //     community_id: inserted_community.id,
  //     user_id: inserted_mod_user.id,
  //   };
  //   let _inserted_moderator = CommunityModerator::join(&conn, &new_moderator);
  //
  //   assert!(!inserted_notmod_user
  //     .is_moderator(&conn, inserted_community.id)
  //     .unwrap());
  //   assert!(inserted_mod_user
  //     .is_moderator(&conn, inserted_community.id)
  //     .unwrap());
  //   assert!(inserted_creator_user
  //     .is_moderator(&conn, inserted_community.id)
  //     .unwrap());
  // }
}
