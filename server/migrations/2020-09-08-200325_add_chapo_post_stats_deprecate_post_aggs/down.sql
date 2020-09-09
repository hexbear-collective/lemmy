drop table hexbear.post_stat cascade;

drop trigger if exists refresh_post_like ON public.post_like;
drop trigger if exists refresh_post ON public.post;
drop trigger if exists refresh_user on public.user_;
drop trigger if exists refresh_comment on public.comment;
drop trigger if exists refresh_community on public.community;

drop function if exists hexbear.refresh_post;
drop function if exists hexbear.refresh_post_like;

create or replace function hexbear.refresh_comment()
    RETURNS trigger
    LANGUAGE 'plpgsql'
AS $BODY$
begin
  IF (TG_OP = 'DELETE') THEN

    -- Update community number of comments
    update community_aggregates_fast as caf
    set number_of_comments = number_of_comments - 1
    from post as p
    where caf.id = p.community_id and p.id = OLD.post_id;

  -- Update hotrank on comment update
  ELSIF (TG_OP = 'UPDATE') THEN

    update hexbear.comment_stat
    set
      hot_rank = hot_rank(coalesce(score, 1)::numeric, (select published from post where id = NEW.post_id)),
      hot_rank_active = hot_rank(coalesce(score, 1)::numeric, NEW.published)
    where comment_id = NEW.id;

  ELSIF (TG_OP = 'INSERT') THEN

    insert into hexbear.comment_stat (comment_id, hot_rank, hot_rank_active)
    values (
      NEW.id,
      hot_rank(0::numeric, (select published from post where id = NEW.post_id)),
      hot_rank(0::numeric, NEW.published)
    );

    -- Update user view due to comment count
    update user_fast 
    set number_of_comments = number_of_comments + 1
    where id = NEW.creator_id;
    
    -- Update post view due to comment count, new comment activity time, but only on new posts
    -- TODO this could be done more efficiently
    delete from post_aggregates_fast where id = NEW.post_id;
    insert into post_aggregates_fast select * from post_aggregates_view where id = NEW.post_id;

    -- Update community number of comments
    update community_aggregates_fast as caf
    set number_of_comments = number_of_comments + 1 
    from post as p
    where caf.id = p.community_id and p.id = NEW.post_id;

  END IF;

  return null;
end $BODY$;

create or replace function hexbear.refresh_user()
    RETURNS trigger
    LANGUAGE 'plpgsql'
AS $BODY$
begin
  IF (TG_OP = 'DELETE') THEN
    delete from user_fast where id = OLD.id;
  ELSIF (TG_OP = 'UPDATE') THEN
    delete from user_fast where id = OLD.id;
    insert into user_fast select * from user_view where id = NEW.id;
    
    -- Refresh post_fast, cause of user info changes
    delete from post_aggregates_fast where creator_id = NEW.id;
    insert into post_aggregates_fast select * from post_aggregates_view where creator_id = NEW.id;

  ELSIF (TG_OP = 'INSERT') THEN
    insert into user_fast select * from user_view where id = NEW.id;
  END IF;

  return null;
end $BODY$;

create or replace function hexbear.refresh_community_user_ban()
    RETURNS trigger
    LANGUAGE 'plpgsql'
AS $BODY$
begin
  -- TODO possibly select from comment_fast to get previous scores, instead of re-fetching the views?
  IF (TG_OP = 'DELETE') THEN
    update post_aggregates_fast set banned_from_community = false where creator_id = OLD.user_id and community_id = OLD.community_id;
  ELSIF (TG_OP = 'INSERT') THEN
    update post_aggregates_fast set banned_from_community = true where creator_id = NEW.user_id and community_id = NEW.community_id;
  END IF;

  return null;
end $BODY$;

create trigger refresh_comment
    after insert or delete or update
    on public.comment
    for each row
    execute procedure hexbear.refresh_comment();

create trigger refresh_post
    after insert or delete or update
    on public.post
    for each row
    execute procedure public.refresh_post();
  
create trigger refresh_post_like
    after insert or delete
    on public.post_like
    for each row
    execute procedure public.refresh_post_like();

create trigger refresh_user
    after insert or delete or update
    on public.user_
    for each row
    execute procedure hexbear.refresh_user();

create trigger refresh_community_user_ban
    after insert or delete
    on public.community_user_ban
    for each row
    execute procedure hexbear.refresh_community_user_ban();

create trigger refresh_community
    after insert or delete or update
    on public.community
    for each row
    execute procedure public.refresh_community();

insert into post_aggregates_fast
select * from post_aggregates_view
on conflict (id) do update 
set score = EXCLUDED.score, upvotes = EXCLUDED.upvotes, 
downvotes = EXCLUDED.downvotes, hot_rank = EXCLUDED.hot_rank, 
hot_rank_active = EXCLUDED.hot_rank_active,
number_of_comments = EXCLUDED.number_of_comments,
newest_activity_time = EXCLUDED.newest_activity_time;