use crate::structs::ModRemoveCommunityView;
use diesel::{result::Error, *};
use lemmy_db_schema::{
  newtypes::PersonId,
  schema::{community, mod_remove_community, person},
  source::{
    community::{Community, CommunitySafe},
    moderator::ModRemoveCommunity,
    person::{Person, PersonSafe},
  },
  traits::{ToSafe, ViewToVec},
  utils::limit_and_offset,
};

type ModRemoveCommunityTuple = (ModRemoveCommunity, PersonSafe, CommunitySafe);

impl ModRemoveCommunityView {
  pub fn list(
    conn: &PgConnection,
    mod_person_id: Option<PersonId>,
    page: Option<i64>,
    limit: Option<i64>,
    hide_mod_names: bool,
  ) -> Result<Vec<Self>, Error> {
    let mut query = mod_remove_community::table
      .inner_join(person::table)
      .inner_join(community::table)
      .select((
        mod_remove_community::all_columns,
        Person::safe_columns_tuple(),
        Community::safe_columns_tuple(),
      ))
      .into_boxed();

    if let Some(mod_person_id) = mod_person_id {
      query = query.filter(mod_remove_community::mod_person_id.eq(mod_person_id));
    };

    let (limit, offset) = limit_and_offset(page, limit);

    let res = query
      .limit(limit)
      .offset(offset)
      .order_by(mod_remove_community::when_.desc())
      .load::<ModRemoveCommunityTuple>(conn)?;

    let mut results = Self::from_tuple_to_vec(res);
    if hide_mod_names {
      results.iter_mut().for_each(|item| {
        item.moderator = None;
      })
    }
    Ok(results)
  }
}

impl ViewToVec for ModRemoveCommunityView {
  type DbTuple = ModRemoveCommunityTuple;
  fn from_tuple_to_vec(items: Vec<Self::DbTuple>) -> Vec<Self> {
    items
      .iter()
      .map(|a| Self {
        mod_remove_community: a.0.to_owned(),
        moderator: Some(a.1.to_owned()),
        community: a.2.to_owned(),
      })
      .collect::<Vec<Self>>()
  }
}
