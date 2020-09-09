-- Replace the post_aggregate table with a 1:1 table solely to house aggregate values
create table hexbear.post_stat
(
    post_id  int references public.post on update cascade on delete cascade primary key,
    number_of_comments int not null default 0,
    score int not null default 0,
    upvotes int not null default 0,
    downvotes int not null default 0,
    hot_rank int,
    hot_rank_active int,
    newest_activity_time timestamp without time zone not null
);

-- Create new trigger functions under 'hexbear' for altered functionality
create or replace function hexbear.refresh_post()
  returns trigger
  language 'plpgsql'
as $BODY$
begin
IF (TG_OP = 'DELETE') THEN
    -- Update community number of posts
    update community_aggregates_fast set number_of_posts = number_of_posts - 1 where id = OLD.community_id;
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
    update community_aggregates_fast set number_of_posts = number_of_posts + 1 where id = NEW.community_id;
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
  ELSIF (TG_OP = 'INSERT') THEN
    insert into user_fast select * from user_view where id = NEW.id;
  END IF;

  return null;
end $BODY$;

create or replace function hexbear.refresh_community()
    RETURNS trigger
    LANGUAGE 'plpgsql'
AS $BODY$
begin
  IF (TG_OP = 'DELETE') THEN
    delete from community_aggregates_fast where id = OLD.id;
  ELSIF (TG_OP = 'UPDATE') THEN
    delete from community_aggregates_fast where id = OLD.id;
    insert into community_aggregates_fast select * from community_aggregates_view where id = NEW.id;

    -- Update user view due to owner changes
    delete from user_fast where id = NEW.creator_id;
    insert into user_fast select * from user_view where id = NEW.creator_id;
    
  -- TODO make sure this shows up in the users page ?
  ELSIF (TG_OP = 'INSERT') THEN
    insert into community_aggregates_fast select * from community_aggregates_view where id = NEW.id;
  END IF;

  return null;
end $BODY$;

drop trigger if exists refresh_post_like ON public.post_like;
drop trigger if exists refresh_post ON public.post;
drop trigger if exists refresh_user on public.user_;
drop trigger if exists refresh_comment on public.comment;
drop trigger if exists refresh_community on public.community;
-- This one stays dropped as we arent "refreshing" duplicate data anymore
drop trigger if exists refresh_community_user_ban on public.community_user_ban;

-- Migrate stats (warning: could take time pending instance size, consider downtime)
insert into hexbear.post_stat (post_id, number_of_comments, score, upvotes, downvotes, hot_rank, hot_rank_active, newest_activity_time)
select id, number_of_comments, score, upvotes, downvotes, hot_rank, hot_rank_active, newest_activity_time from post_aggregates_fast;

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

create trigger refresh_post
    after insert or delete or update
    on public.post
    for each row
    execute procedure hexbear.refresh_post();

create trigger refresh_user
    after insert or delete or update
    on public.user_
    for each row
    execute procedure hexbear.refresh_user();

create trigger refresh_community
    after insert or delete or update
    on public.community
    for each row
    execute procedure hexbear.refresh_community();

CREATE OR REPLACE VIEW hexbear.post_fast_view
 AS
 SELECT pav.*,
    us.id AS user_id,
    us.user_vote AS my_vote,
    us.is_subbed::boolean AS subscribed,
    us.is_read::boolean AS read,
    us.is_saved::boolean AS saved
   FROM (SELECT p.id,
    p.name,
    p.url,
    p.body,
    p.creator_id,
    p.community_id,
    p.removed,
    p.locked,
    p.published,
    p.updated,
    p.deleted,
    p.nsfw,
    p.stickied,
    p.embed_title,
    p.embed_description,
    p.embed_html,
    p.thumbnail_url,
    p.ap_id,
    p.local,
    u.actor_id AS creator_actor_id,
    u.local AS creator_local,
    u.name AS creator_name,
    u.preferred_username AS creator_preferred_username,
    u.published AS creator_published,
    u.avatar AS creator_avatar,
    ut.tags AS creator_tags,
    cut.tags AS creator_community_tags,
    u.banned,
    cb.id::boolean AS banned_from_community,
    c.actor_id AS community_actor_id,
    c.local AS community_local,
    c.name AS community_name,
    c.icon AS community_icon,
    c.removed AS community_removed,
    c.deleted AS community_deleted,
    c.nsfw AS community_nsfw,
    COALESCE(cps.number_of_comments, 0)::bigint AS number_of_comments,
    COALESCE(cps.score, 0)::bigint AS score,
    COALESCE(cps.upvotes, 0)::bigint AS upvotes,
    COALESCE(cps.downvotes, 0)::bigint AS downvotes,
    COALESCE(cps.hot_rank, 0) AS hot_rank,
    COALESCE(cps.hot_rank_active, 0) AS hot_rank_active,
    COALESCE(cps.newest_activity_time, p.published) AS newest_activity_time
   FROM post p
     LEFT JOIN user_ u ON p.creator_id = u.id
     LEFT JOIN user_tag ut ON p.creator_id = ut.user_id
     LEFT JOIN community_user_tag cut ON p.creator_id = cut.user_id AND p.community_id = cut.community_id
     LEFT JOIN community_user_ban cb ON p.creator_id = cb.user_id AND p.community_id = cb.community_id
     LEFT JOIN community c ON p.community_id = c.id
     LEFT JOIN hexbear.post_stat cps on cps.post_id = p.id
	) pav
     CROSS JOIN LATERAL ( SELECT u.id,
            COALESCE(cf.community_id, 0) AS is_subbed,
            COALESCE(pr.post_id, 0) AS is_read,
            COALESCE(ps.post_id, 0) AS is_saved,
            COALESCE(pl.score::integer, 0) AS user_vote
           FROM user_ u
             LEFT JOIN community_user_ban cb ON u.id = cb.user_id AND cb.community_id = pav.community_id
             LEFT JOIN community_follower cf ON u.id = cf.user_id AND cf.community_id = pav.community_id
             LEFT JOIN post_read pr ON u.id = pr.user_id AND pr.post_id = pav.id
             LEFT JOIN post_saved ps ON u.id = ps.user_id AND ps.post_id = pav.id
             LEFT JOIN post_like pl ON u.id = pl.user_id AND pav.id = pl.post_id) us
UNION ALL
SELECT p.id,
  p.name,
  p.url,
  p.body,
  p.creator_id,
  p.community_id,
  p.removed,
  p.locked,
  p.published,
  p.updated,
  p.deleted,
  p.nsfw,
  p.stickied,
  p.embed_title,
  p.embed_description,
  p.embed_html,
  p.thumbnail_url,
  p.ap_id,
  p.local,
  u.actor_id AS creator_actor_id,
  u.local AS creator_local,
  u.name AS creator_name,
  u.preferred_username AS creator_preferred_username,
  u.published AS creator_published,
  u.avatar AS creator_avatar,
  ut.tags AS creator_tags,
  cut.tags AS creator_community_tags,
  u.banned,
  cb.id::boolean AS banned_from_community,
  c.actor_id AS community_actor_id,
  c.local AS community_local,
  c.name AS community_name,
  c.icon AS community_icon,
  c.removed AS community_removed,
  c.deleted AS community_deleted,
  c.nsfw AS community_nsfw,
  COALESCE(cps.number_of_comments, 0)::bigint AS number_of_comments,
  COALESCE(cps.score, 0)::bigint AS score,
  COALESCE(cps.upvotes, 0)::bigint AS upvotes,
  COALESCE(cps.downvotes, 0)::bigint AS downvotes,
  COALESCE(cps.hot_rank, 0) AS hot_rank,
  COALESCE(cps.hot_rank_active, 0) AS hot_rank_active,
  COALESCE(cps.newest_activity_time, p.published) AS newest_activity_time,
  null::integer AS user_id,
  null::integer AS my_vote,
  null::boolean AS subscribed,
  null::boolean AS read,
  null::boolean AS saved
  FROM post p
    LEFT JOIN user_ u ON p.creator_id = u.id
    LEFT JOIN user_tag ut ON p.creator_id = ut.user_id
    LEFT JOIN community_user_tag cut ON p.creator_id = cut.user_id AND p.community_id = cut.community_id
    LEFT JOIN community_user_ban cb ON p.creator_id = cb.user_id AND p.community_id = cb.community_id
    LEFT JOIN community c ON p.community_id = c.id
    LEFT JOIN hexbear.post_stat cps on cps.post_id = p.id;