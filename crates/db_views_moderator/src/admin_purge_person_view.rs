use crate::structs::AdminPurgePersonView;
use diesel::{result::Error, *};
use lemmy_db_schema::{
  newtypes::PersonId,
  schema::{admin_purge_person, person},
  source::{
    moderator::AdminPurgePerson,
    person::{Person, PersonSafe},
  },
  traits::{ToSafe, ViewToVec},
  utils::limit_and_offset,
};

type AdminPurgePersonViewTuple = (AdminPurgePerson, PersonSafe);

impl AdminPurgePersonView {
  pub fn list(
    conn: &PgConnection,
    admin_person_id: Option<PersonId>,
    page: Option<i64>,
    limit: Option<i64>,
    hide_mod_names: bool,
  ) -> Result<Vec<Self>, Error> {
    let mut query = admin_purge_person::table
      .inner_join(person::table.on(admin_purge_person::admin_person_id.eq(person::id)))
      .select((
        admin_purge_person::all_columns,
        Person::safe_columns_tuple(),
      ))
      .into_boxed();

    if let Some(admin_person_id) = admin_person_id {
      query = query.filter(admin_purge_person::admin_person_id.eq(admin_person_id));
    };

    let (limit, offset) = limit_and_offset(page, limit);

    let res = query
      .limit(limit)
      .offset(offset)
      .order_by(admin_purge_person::when_.desc())
      .load::<AdminPurgePersonViewTuple>(conn)?;

    let mut results = Self::from_tuple_to_vec(res);
    if hide_mod_names {
      results.iter_mut().for_each(|item| {
        item.admin = None;
      })
    }
    Ok(results)
  }
}

impl ViewToVec for AdminPurgePersonView {
  type DbTuple = AdminPurgePersonViewTuple;
  fn from_tuple_to_vec(items: Vec<Self::DbTuple>) -> Vec<Self> {
    items
      .iter()
      .map(|a| Self {
        admin_purge_person: a.0.to_owned(),
        admin: Some(a.1.to_owned()),
      })
      .collect::<Vec<Self>>()
  }
}
