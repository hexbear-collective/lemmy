use crate::structs::ModAddView;
use diesel::{result::Error, *};
use lemmy_db_schema::{
  newtypes::PersonId,
  schema::{mod_add, person, person_alias_1},
  source::{
    moderator::ModAdd,
    person::{Person, PersonAlias1, PersonSafe, PersonSafeAlias1},
  },
  traits::{ToSafe, ViewToVec},
  utils::limit_and_offset,
};

type ModAddViewTuple = (ModAdd, PersonSafe, PersonSafeAlias1);

impl ModAddView {
  pub fn list(
    conn: &PgConnection,
    mod_person_id: Option<PersonId>,
    other_person_id: Option<PersonId>,
    page: Option<i64>,
    limit: Option<i64>,
    hide_mod_names: bool,
  ) -> Result<Vec<Self>, Error> {
    let mut query = mod_add::table
      .inner_join(person::table.on(mod_add::mod_person_id.eq(person::id)))
      .inner_join(person_alias_1::table.on(mod_add::other_person_id.eq(person_alias_1::id)))
      .select((
        mod_add::all_columns,
        Person::safe_columns_tuple(),
        PersonAlias1::safe_columns_tuple(),
      ))
      .into_boxed();

    if let Some(mod_person_id) = mod_person_id {
      query = query.filter(mod_add::mod_person_id.eq(mod_person_id));
    };

    if let Some(other_person_id) = other_person_id {
      query = query.filter(person_alias_1::id.eq(other_person_id));
    };

    let (limit, offset) = limit_and_offset(page, limit);

    let mut res = query
      .limit(limit)
      .offset(offset)
      .order_by(mod_add::when_.desc())
      .load::<ModAddViewTuple>(conn)?;

    if hide_mod_names {
      res.iter_mut().for_each(|item| {
        item.1.name.clear();
      });
    }
    Ok(Self::from_tuple_to_vec(res))
  }
}

impl ViewToVec for ModAddView {
  type DbTuple = ModAddViewTuple;
  fn from_tuple_to_vec(items: Vec<Self::DbTuple>) -> Vec<Self> {
    items
      .iter()
      .map(|a| Self {
        mod_add: a.0.to_owned(),
        moderator: a.1.to_owned(),
        modded_person: a.2.to_owned(),
      })
      .collect::<Vec<Self>>()
  }
}
