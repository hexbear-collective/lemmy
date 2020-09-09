drop table hexbear.user_stat cascade;

drop trigger if exists refresh_post_like ON public.post_like;
drop trigger if exists refresh_post ON public.post;
drop trigger if exists refresh_user on public.user_;
drop trigger if exists refresh_comment on public.comment;
drop trigger if exists refresh_community on public.community;
drop trigger if exists refresh_comment_like ON public.comment_like;

create or replace function hexbear.refresh_post()
  returns trigger
  language 'plpgsql'
as $BODY$
begin
IF (TG_OP = 'DELETE') THEN
    -- Update community number of posts
    update hexbear.community_stat set number_of_posts = number_of_posts - 1 where community_id = OLD.community_id;
  ELSIF (TG_OP = 'UPDATE') THEN
  -- only hot ranks actually depend on a post update
    update hexbear.post_stat
    set
      hot_rank = hot_rank(coalesce(score, 1::bigint)::numeric, NEW.published),
      hot_rank_active = hot_rank(coalesce(score, 1::bigint)::numeric, 
        NEW.published + '24:00:00'::interval * (1::double precision - exp('-0.000012146493725346809'::numeric::double precision * 
        date_part('epoch'::text, GREATEST(newest_activity_time, NEW.published) - NEW.published
      ))))
    where post_id = NEW.id;

  ELSIF (TG_OP = 'INSERT') THEN

    insert into hexbear.post_stat (post_id, hot_rank, hot_rank_active, newest_activity_time)
    values (
      NEW.id,
      hot_rank(0::numeric, NEW.published),
      hot_rank(0::numeric, 
        NEW.published + '24:00:00'::interval * (1::double precision - exp('-0.000012146493725346809'::numeric::double precision * 
        date_part('epoch'::text, NEW.published - NEW.published)
      ))),
      NEW.published
    );

    -- Update that users number of posts, post score
    delete from user_fast where id = NEW.creator_id;
    insert into user_fast select * from user_view where id = NEW.creator_id;
  
    -- Update community number of posts
    update hexbear.community_stat set number_of_posts = number_of_posts + 1 where community_id = NEW.community_id;
  END IF;

  return null;
end $BODY$;

create or replace function hexbear.refresh_comment()
    RETURNS trigger
    LANGUAGE 'plpgsql'
AS $BODY$
begin
  IF (TG_OP = 'DELETE') THEN

    -- Update community number of comments
    update hexbear.community_stat as ccs
    set number_of_comments = number_of_comments - 1
    from post as p
    where ccs.community_id = p.community_id and p.id = OLD.post_id;

    update hexbear.post_stat
    set
      number_of_comments = number_of_comments - 1
    where post_id = OLD.post_id;

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

    update hexbear.post_stat
    set
      number_of_comments = number_of_comments + 1,
      newest_activity_time = NEW.published
    where post_id = NEW.post_id;

    -- Update user view due to comment count
    update user_fast 
    set number_of_comments = number_of_comments + 1
    where id = NEW.creator_id;

    -- Update community number of comments
    update hexbear.community_stat as ccs
    set number_of_comments = number_of_comments + 1
    from post as p
    where ccs.community_id = p.community_id and p.id = NEW.post_id;

  END IF;

  return null;
end $BODY$;

create or replace function hexbear.refresh_community()
    RETURNS trigger
    LANGUAGE 'plpgsql'
AS $BODY$
begin
  IF (TG_OP = 'UPDATE') THEN

    update hexbear.community_stat
    set
      hot_rank = hot_rank(number_of_subs::numeric, NEW.published)
    where community_id = NEW.id;
    -- Update user view due to owner changes
    delete from user_fast where id = NEW.creator_id;
    insert into user_fast select * from user_view where id = NEW.creator_id;
    
  -- TODO make sure this shows up in the users page ?
  ELSIF (TG_OP = 'INSERT') THEN
    insert into hexbear.community_stat (community_id, hot_rank)
    values (
      NEW.id,
      hot_rank(0::numeric, NEW.published)
    );
  END IF;

  return null;
end $BODY$;

create or replace function hexbear.refresh_post_like()
    RETURNS trigger
    LANGUAGE 'plpgsql'
AS $BODY$
begin
  IF (TG_OP = 'DELETE') THEN

    update hexbear.post_stat
    set score = score - OLD.score,
    upvotes = case 
      when (OLD.score = 1) then upvotes - 1 
      else upvotes end,
    downvotes = case 
      when (OLD.score = -1) then downvotes - 1 
      else downvotes end
    where post_id = OLD.post_id;

  ELSIF (TG_OP = 'INSERT') THEN

    update hexbear.post_stat
    set score = score + NEW.score,
    upvotes = case 
      when (NEW.score = 1) then upvotes + 1 
      else upvotes end,
    downvotes = case 
      when (NEW.score = -1) then downvotes + 1 
      else downvotes end
    where post_id = NEW.post_id;

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
  ELSIF (TG_OP = 'INSERT') THEN
    insert into user_fast select * from user_view where id = NEW.id;
  END IF;

  return null;
end $BODY$;

create or replace function hexbear.refresh_comment_like()
    RETURNS trigger
    LANGUAGE 'plpgsql'
AS $BODY$
begin
  IF (TG_OP = 'DELETE') THEN
    update hexbear.comment_stat
    set score = score - OLD.score,
    upvotes = case 
      when (OLD.score = 1) then upvotes - 1 
      else upvotes end,
    downvotes = case 
      when (OLD.score = -1) then downvotes - 1 
      else downvotes end
    where comment_id = OLD.comment_id;

  ELSIF (TG_OP = 'INSERT') THEN
    update hexbear.comment_stat
    set score = score + NEW.score,
    upvotes = case 
      when (NEW.score = 1) then upvotes + 1 
      else upvotes end,
    downvotes = case 
      when (NEW.score = -1) then downvotes + 1 
      else downvotes end
    where comment_id = NEW.comment_id;
  END IF;

  return null;
end $BODY$;

-- Migrate stats (warning: could take time pending instance size, consider downtime)
insert into user_fast
select * from public.user_view
on conflict (id) do update 
set 
number_of_comments = EXCLUDED.number_of_comments,
number_of_posts = EXCLUDED.number_of_posts,
post_score = EXCLUDED.post_score,
comment_score = EXCLUDED.comment_score;

-- Add new triggers
create trigger refresh_comment
    after insert or delete or update
    on public.comment
    for each row
    execute procedure hexbear.refresh_comment();
  
create trigger refresh_post_like
    after insert or delete
    on public.post_like
    for each row
    execute procedure hexbear.refresh_post_like();

create trigger refresh_comment_like
    after insert or delete
    on public.comment_like
    for each row
    execute procedure hexbear.refresh_comment_like();

create trigger refresh_post
    after insert or delete or update
    on public.post
    for each row
    execute procedure hexbear.refresh_post();

create trigger refresh_user
    after insert
    on public.user_
    for each row
    execute procedure hexbear.refresh_user();

create trigger refresh_community
    after insert or update
    on public.community
    for each row
    execute procedure hexbear.refresh_community();
    