use crate::structs::ModLockPostView;
use diesel::{result::Error, *};
use lemmy_db_schema::{
  newtypes::{CommunityId, PersonId},
  schema::{community, mod_lock_post, person, person_alias_1, post},
  source::{
    community::{Community, CommunitySafe},
    moderator::ModLockPost,
    person::{Person, PersonSafe},
    post::Post,
  },
  traits::{ToSafe, ViewToVec},
  utils::limit_and_offset,
};

type ModLockPostViewTuple = (ModLockPost, PersonSafe, Post, CommunitySafe);

impl ModLockPostView {
  pub fn list(
    conn: &PgConnection,
    community_id: Option<CommunityId>,
    mod_person_id: Option<PersonId>,
    other_person_id: Option<PersonId>,
    page: Option<i64>,
    limit: Option<i64>,
    hide_mod_names: bool,
  ) -> Result<Vec<Self>, Error> {
    let mut query = mod_lock_post::table
      .inner_join(person::table)
      .inner_join(post::table)
      .inner_join(community::table.on(post::community_id.eq(community::id)))
      .inner_join(person_alias_1::table.on(post::creator_id.eq(person_alias_1::id)))
      .select((
        mod_lock_post::all_columns,
        Person::safe_columns_tuple(),
        post::all_columns,
        Community::safe_columns_tuple(),
      ))
      .into_boxed();

    if let Some(community_id) = community_id {
      query = query.filter(post::community_id.eq(community_id));
    };

    if let Some(mod_person_id) = mod_person_id {
      query = query.filter(mod_lock_post::mod_person_id.eq(mod_person_id));
    };

    if let Some(other_person_id) = other_person_id {
      query = query.filter(person_alias_1::id.eq(other_person_id));
    };

    let (limit, offset) = limit_and_offset(page, limit);

    let mut res = query
      .limit(limit)
      .offset(offset)
      .order_by(mod_lock_post::when_.desc())
      .load::<ModLockPostViewTuple>(conn)?;

    if hide_mod_names {
      res.iter_mut().for_each(|item| {
        item.1.name.clear();
      });
    }
    Ok(Self::from_tuple_to_vec(res))
  }
}

impl ViewToVec for ModLockPostView {
  type DbTuple = ModLockPostViewTuple;
  fn from_tuple_to_vec(items: Vec<Self::DbTuple>) -> Vec<Self> {
    items
      .iter()
      .map(|a| Self {
        mod_lock_post: a.0.to_owned(),
        moderator: a.1.to_owned(),
        post: a.2.to_owned(),
        community: a.3.to_owned(),
      })
      .collect::<Vec<Self>>()
  }
}
