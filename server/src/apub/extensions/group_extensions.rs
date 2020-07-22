use crate::LemmyError;
use activitystreams_ext::UnparsedExtension;
use activitystreams_new::unparsed::UnparsedMutExt;
use diesel::PgConnection;
use lemmy_db::{category::Category, Crud};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupExtension {
  pub category: GroupCategory,
  pub sensitive: bool,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupCategory {
  // Using a string because that's how Peertube does it.
  pub identifier: String,
  pub name: String,
}

impl GroupExtension {
  pub fn new(
    conn: &PgConnection,
    category_id: i32,
    sensitive: bool,
  ) -> Result<GroupExtension, LemmyError> {
    let category = Category::read(conn, category_id)?;
    let group_category = GroupCategory {
      identifier: category_id.to_string(),
      name: category.name,
    };
    Ok(GroupExtension {
      category: group_category,
      sensitive,
    })
  }
}

impl<U> UnparsedExtension<U> for GroupExtension
where
  U: UnparsedMutExt,
{
  type Error = serde_json::Error;

  fn try_from_unparsed(unparsed_mut: &mut U) -> Result<Self, Self::Error> {
    Ok(GroupExtension {
      category: unparsed_mut.remove("category")?,
      sensitive: unparsed_mut.remove("sensitive")?,
    })
  }

  fn try_into_unparsed(self, unparsed_mut: &mut U) -> Result<(), Self::Error> {
    unparsed_mut.insert("category", self.category)?;
    unparsed_mut.insert("sensitive", self.sensitive)?;
    Ok(())
  }
}
