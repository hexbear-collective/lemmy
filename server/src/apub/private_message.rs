use crate::{
  apub::{
<<<<<<< HEAD
    activities::{generate_activity_id, send_activity},
    create_tombstone,
    fetcher::get_or_fetch_and_upsert_user,
    insert_activity,
=======
    activities::generate_activity_id,
    activity_queue::send_activity,
    check_actor_domain,
    check_is_apub_id_valid,
    create_tombstone,
    fetcher::get_or_fetch_and_upsert_user,
    insert_activity,
    ActorType,
>>>>>>> 11149ba0
    ApubObjectType,
    FromApub,
    ToApub,
  },
  blocking,
  DbPool,
<<<<<<< HEAD
  LemmyError,
=======
  LemmyContext,
>>>>>>> 11149ba0
};
use activitystreams::{
  activity::{
    kind::{CreateType, DeleteType, UndoType, UpdateType},
    Create,
    Delete,
    Undo,
    Update,
  },
<<<<<<< HEAD
  context,
=======
>>>>>>> 11149ba0
  object::{kind::NoteType, Note, Tombstone},
  prelude::*,
};
use anyhow::Context;
use lemmy_db::{
  private_message::{PrivateMessage, PrivateMessageForm},
  user::User_,
  Crud,
};
use lemmy_utils::{convert_datetime, location_info, LemmyError};
use url::Url;

#[async_trait::async_trait(?Send)]
impl ToApub for PrivateMessage {
  type Response = Note;

  async fn to_apub(&self, pool: &DbPool) -> Result<Note, LemmyError> {
    let mut private_message = Note::new();

    let creator_id = self.creator_id;
    let creator = blocking(pool, move |conn| User_::read(conn, creator_id)).await??;

    let recipient_id = self.recipient_id;
    let recipient = blocking(pool, move |conn| User_::read(conn, recipient_id)).await??;

    private_message
      .set_context(activitystreams::context())
      .set_id(Url::parse(&self.ap_id.to_owned())?)
      .set_published(convert_datetime(self.published))
      .set_content(self.content.to_owned())
      .set_to(recipient.actor_id)
      .set_attributed_to(creator.actor_id);

    if let Some(u) = self.updated {
      private_message.set_updated(convert_datetime(u));
    }

    Ok(private_message)
  }

  fn to_tombstone(&self) -> Result<Tombstone, LemmyError> {
    create_tombstone(self.deleted, &self.ap_id, self.updated, NoteType::Note)
  }
}

#[async_trait::async_trait(?Send)]
impl FromApub for PrivateMessageForm {
  type ApubType = Note;

  /// Parse an ActivityPub note received from another instance into a Lemmy Private message
  async fn from_apub(
    note: &Note,
<<<<<<< HEAD
    client: &Client,
    pool: &DbPool,
=======
    context: &LemmyContext,
    expected_domain: Option<Url>,
>>>>>>> 11149ba0
  ) -> Result<PrivateMessageForm, LemmyError> {
    let creator_actor_id = note
      .attributed_to()
      .context(location_info!())?
      .clone()
      .single_xsd_any_uri()
<<<<<<< HEAD
      .unwrap();

    let creator = get_or_fetch_and_upsert_user(&creator_actor_id, client, pool).await?;

    let recipient_actor_id = note.to().unwrap().clone().single_xsd_any_uri().unwrap();

    let recipient = get_or_fetch_and_upsert_user(&recipient_actor_id, client, pool).await?;
=======
      .context(location_info!())?;

    let creator = get_or_fetch_and_upsert_user(&creator_actor_id, context).await?;
    let recipient_actor_id = note
      .to()
      .context(location_info!())?
      .clone()
      .single_xsd_any_uri()
      .context(location_info!())?;
    let recipient = get_or_fetch_and_upsert_user(&recipient_actor_id, context).await?;
    let ap_id = note.id_unchecked().context(location_info!())?.to_string();
    check_is_apub_id_valid(&Url::parse(&ap_id)?)?;
>>>>>>> 11149ba0

    Ok(PrivateMessageForm {
      creator_id: creator.id,
      recipient_id: recipient.id,
      content: note
        .content()
        .context(location_info!())?
        .as_single_xsd_string()
        .context(location_info!())?
        .to_string(),
      published: note.published().map(|u| u.to_owned().naive_local()),
      updated: note.updated().map(|u| u.to_owned().naive_local()),
      deleted: None,
      read: None,
<<<<<<< HEAD
      ap_id: note.id_unchecked().unwrap().to_string(),
=======
      ap_id: Some(check_actor_domain(note, expected_domain)?),
>>>>>>> 11149ba0
      local: false,
    })
  }
}

#[async_trait::async_trait(?Send)]
impl ApubObjectType for PrivateMessage {
  /// Send out information about a newly created private message
<<<<<<< HEAD
  async fn send_create(
    &self,
    creator: &User_,
    client: &Client,
    pool: &DbPool,
  ) -> Result<(), LemmyError> {
    let note = self.to_apub(pool).await?;
=======
  async fn send_create(&self, creator: &User_, context: &LemmyContext) -> Result<(), LemmyError> {
    let note = self.to_apub(context.pool()).await?;
>>>>>>> 11149ba0

    let recipient_id = self.recipient_id;
    let recipient = blocking(context.pool(), move |conn| User_::read(conn, recipient_id)).await??;

    let mut create = Create::new(creator.actor_id.to_owned(), note.into_any_base()?);
    let to = recipient.get_inbox_url()?;
    create
<<<<<<< HEAD
      .set_context(context())
=======
      .set_context(activitystreams::context())
>>>>>>> 11149ba0
      .set_id(generate_activity_id(CreateType::Create)?)
      .set_to(to.clone());

    insert_activity(creator.id, create.clone(), true, context.pool()).await?;

    send_activity(context.activity_queue(), create, creator, vec![to])?;
    Ok(())
  }

  /// Send out information about an edited post, to the followers of the community.
<<<<<<< HEAD
  async fn send_update(
    &self,
    creator: &User_,
    client: &Client,
    pool: &DbPool,
  ) -> Result<(), LemmyError> {
    let note = self.to_apub(pool).await?;
=======
  async fn send_update(&self, creator: &User_, context: &LemmyContext) -> Result<(), LemmyError> {
    let note = self.to_apub(context.pool()).await?;
>>>>>>> 11149ba0

    let recipient_id = self.recipient_id;
    let recipient = blocking(context.pool(), move |conn| User_::read(conn, recipient_id)).await??;

    let mut update = Update::new(creator.actor_id.to_owned(), note.into_any_base()?);
    let to = recipient.get_inbox_url()?;
    update
<<<<<<< HEAD
      .set_context(context())
=======
      .set_context(activitystreams::context())
>>>>>>> 11149ba0
      .set_id(generate_activity_id(UpdateType::Update)?)
      .set_to(to.clone());

    insert_activity(creator.id, update.clone(), true, context.pool()).await?;

    send_activity(context.activity_queue(), update, creator, vec![to])?;
    Ok(())
  }

<<<<<<< HEAD
  async fn send_delete(
    &self,
    creator: &User_,
    client: &Client,
    pool: &DbPool,
  ) -> Result<(), LemmyError> {
    let note = self.to_apub(pool).await?;
=======
  async fn send_delete(&self, creator: &User_, context: &LemmyContext) -> Result<(), LemmyError> {
    let note = self.to_apub(context.pool()).await?;
>>>>>>> 11149ba0

    let recipient_id = self.recipient_id;
    let recipient = blocking(context.pool(), move |conn| User_::read(conn, recipient_id)).await??;

    let mut delete = Delete::new(creator.actor_id.to_owned(), note.into_any_base()?);
    let to = recipient.get_inbox_url()?;
    delete
<<<<<<< HEAD
      .set_context(context())
=======
      .set_context(activitystreams::context())
>>>>>>> 11149ba0
      .set_id(generate_activity_id(DeleteType::Delete)?)
      .set_to(to.clone());

    insert_activity(creator.id, delete.clone(), true, context.pool()).await?;

    send_activity(context.activity_queue(), delete, creator, vec![to])?;
    Ok(())
  }

  async fn send_undo_delete(
    &self,
    creator: &User_,
    context: &LemmyContext,
  ) -> Result<(), LemmyError> {
<<<<<<< HEAD
    let note = self.to_apub(pool).await?;
=======
    let note = self.to_apub(context.pool()).await?;
>>>>>>> 11149ba0

    let recipient_id = self.recipient_id;
    let recipient = blocking(context.pool(), move |conn| User_::read(conn, recipient_id)).await??;

    let mut delete = Delete::new(creator.actor_id.to_owned(), note.into_any_base()?);
    let to = recipient.get_inbox_url()?;
    delete
<<<<<<< HEAD
      .set_context(context())
=======
      .set_context(activitystreams::context())
>>>>>>> 11149ba0
      .set_id(generate_activity_id(DeleteType::Delete)?)
      .set_to(to.clone());

    // Undo that fake activity
    let mut undo = Undo::new(creator.actor_id.to_owned(), delete.into_any_base()?);
    undo
<<<<<<< HEAD
      .set_context(context())
=======
      .set_context(activitystreams::context())
>>>>>>> 11149ba0
      .set_id(generate_activity_id(UndoType::Undo)?)
      .set_to(to.clone());

    insert_activity(creator.id, undo.clone(), true, context.pool()).await?;

    send_activity(context.activity_queue(), undo, creator, vec![to])?;
    Ok(())
  }

  async fn send_remove(&self, _mod_: &User_, _context: &LemmyContext) -> Result<(), LemmyError> {
    unimplemented!()
  }

  async fn send_undo_remove(
    &self,
    _mod_: &User_,
    _context: &LemmyContext,
  ) -> Result<(), LemmyError> {
    unimplemented!()
  }
}
