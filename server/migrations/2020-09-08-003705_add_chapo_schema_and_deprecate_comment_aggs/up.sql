create schema hexbear;

-- Replace the comment_aggregate table with a 1:1 table solely to house aggregate values
create table hexbear.comment_stat
(
    comment_id  int references public.comment on update cascade on delete cascade primary key,
    score int not null default 0,
    upvotes int not null default 0,
    downvotes int not null default 0,
    hot_rank int,
    hot_rank_active int
);

-- Create new trigger functions under 'hexbear' for altered functionality
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

-- Drop existing triggers
drop trigger if exists refresh_comment_like ON public.comment_like;
drop trigger if exists refresh_comment ON public.comment;
drop trigger if exists refresh_user on public.user_;
drop trigger if exists refresh_community_user_ban on public.community_user_ban;

-- Migrate stats (warning: could take time pending instance size, consider downtime)
insert into hexbear.comment_stat (comment_id, score, upvotes, downvotes, hot_rank, hot_rank_active)
select id, score, upvotes, downvotes, hot_rank, hot_rank_active from comment_aggregates_fast;

-- Add new triggers
create trigger refresh_comment
    after insert or delete or update
    on public.comment
    for each row
    execute procedure hexbear.refresh_comment();
  
create trigger refresh_comment_like
    after insert or delete
    on public.comment_like
    for each row
    execute procedure hexbear.refresh_comment_like();

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

-- Add altered view, this one combines the "aggregate view" into the normal view
create or replace view hexbear.comment_fast_view as

SELECT
 	cav.*,
	us.*
FROM (
	select 
		ct.id,
		ct.creator_id,
		ct.post_id,
		ct.parent_id,
		ct.content,
		ct.removed,
		ct.read,
		ct.published,
		ct.updated,
		ct.deleted,
		ct.ap_id,
		ct.local,
		p.name AS post_name,
		p.community_id,
		c.actor_id AS community_actor_id,
		c.local AS community_local,
		c.name AS community_name,
		c.icon AS community_icon,
		u.banned,
		COALESCE(cb.id, 0)::boolean AS banned_from_community,
		u.actor_id AS creator_actor_id,
		u.local AS creator_local,
		u.name AS creator_name,
		u.preferred_username AS creator_preferred_username,
		u.published AS creator_published,
		u.avatar AS creator_avatar,
		ut.tags AS creator_tags,
		cut.tags AS creator_community_tags,
		COALESCE(ccs.score, 0)::bigint AS score,
		COALESCE(ccs.upvotes, 0)::bigint AS upvotes,
		COALESCE(ccs.downvotes, 0)::bigint AS downvotes,
	  COALESCE(ccs.hot_rank, 0) as hot_rank,
	  COALESCE(ccs.hot_rank_active, 0) as hot_rank_active
	from comment ct
  LEFT JOIN post p ON ct.post_id = p.id
  LEFT JOIN community c ON p.community_id = c.id
  LEFT JOIN user_ u ON ct.creator_id = u.id
  LEFT JOIN user_tag ut ON ct.creator_id = ut.user_id
  LEFT JOIN community_user_tag cut ON ct.creator_id = cut.user_id AND p.community_id = cut.community_id
  LEFT JOIN community_user_ban cb ON ct.creator_id = cb.user_id AND p.id = ct.post_id AND p.community_id = cb.community_id
	LEFT JOIN hexbear.comment_stat ccs ON ccs.comment_id = ct.id
  ) cav
  cross join lateral ( 
    SELECT u.id AS user_id,
          COALESCE(cl.score::integer, 0) AS my_vote,
          COALESCE(cf.id, 0)::boolean AS subscribed,
          COALESCE(cs.id, 0)::boolean AS saved
      FROM user_ u
            LEFT JOIN comment_like cl ON u.id = cl.user_id AND cl.comment_id = cav.id
            LEFT JOIN comment_saved cs ON u.id = cs.user_id AND cs.comment_id = cav.id
            LEFT JOIN community_follower cf ON u.id = cf.user_id AND cav.community_id = cf.community_id
	) us

union all

select 
		ct.id,
		ct.creator_id,
		ct.post_id,
		ct.parent_id,
		ct.content,
		ct.removed,
		ct.read,
		ct.published,
		ct.updated,
		ct.deleted,
		ct.ap_id,
		ct.local,
		p.name AS post_name,
		p.community_id,
		c.actor_id AS community_actor_id,
		c.local AS community_local,
		c.name AS community_name,
		c.icon AS community_icon,
		u.banned,
		COALESCE(cb.id, 0)::boolean AS banned_from_community,
		u.actor_id AS creator_actor_id,
		u.local AS creator_local,
		u.name AS creator_name,
		u.preferred_username AS creator_preferred_username,
		u.published AS creator_published,
		u.avatar AS creator_avatar,
		ut.tags AS creator_tags,
		cut.tags AS creator_community_tags,
		COALESCE(ccs.score, 0)::bigint AS score,
		COALESCE(ccs.upvotes, 0)::bigint AS upvotes,
		COALESCE(ccs.downvotes, 0)::bigint AS downvotes,
	  COALESCE(ccs.hot_rank, 0) as hot_rank,
	 	COALESCE(ccs.hot_rank_active, 0) as hot_rank_active,
		NULL::integer AS user_id,
		NULL::integer AS my_vote,
		NULL::boolean AS subscribed,
		NULL::boolean AS saved
  from comment ct
  LEFT JOIN post p ON ct.post_id = p.id
  LEFT JOIN community c ON p.community_id = c.id
  LEFT JOIN user_ u ON ct.creator_id = u.id
  LEFT JOIN user_tag ut ON ct.creator_id = ut.user_id
  LEFT JOIN community_user_tag cut ON ct.creator_id = cut.user_id AND p.community_id = cut.community_id
  LEFT JOIN community_user_ban cb ON ct.creator_id = cb.user_id AND p.id = ct.post_id AND p.community_id = cb.community_id
	LEFT JOIN hexbear.comment_stat ccs ON ccs.comment_id = ct.id;

CREATE OR REPLACE VIEW hexbear.reply_fast_view
 AS
 WITH closereply AS (
         SELECT c2.id,
            c2.creator_id AS sender_id,
            c.creator_id AS recipient_id
           FROM comment c
             JOIN comment c2 ON c.id = c2.parent_id
          WHERE c2.creator_id <> c.creator_id
        UNION
         SELECT c.id,
            c.creator_id AS sender_id,
            p.creator_id AS recipient_id
           FROM comment c,
            post p
          WHERE c.post_id = p.id AND c.parent_id IS NULL AND c.creator_id <> p.creator_id
        )
 SELECT cv.id,
    cv.creator_id,
    cv.post_id,
    cv.parent_id,
    cv.content,
    cv.removed,
    cv.read,
    cv.published,
    cv.updated,
    cv.deleted,
    cv.ap_id,
    cv.local,
    cv.post_name,
    cv.community_id,
    cv.community_actor_id,
    cv.community_local,
    cv.community_name,
    cv.community_icon,
    cv.banned,
    cv.banned_from_community,
    cv.creator_actor_id,
    cv.creator_local,
    cv.creator_name,
    cv.creator_preferred_username,
    cv.creator_published,
    cv.creator_avatar,
    cv.creator_tags,
    cv.creator_community_tags,
    cv.score,
    cv.upvotes,
    cv.downvotes,
    cv.hot_rank,
    cv.hot_rank_active,
    cv.user_id,
    cv.my_vote,
    cv.subscribed,
    cv.saved,
    closereply.recipient_id
   FROM hexbear.comment_fast_view cv,
    closereply
  WHERE closereply.id = cv.id;

  
CREATE OR REPLACE VIEW hexbear.user_mention_fast_view
 AS
 SELECT ac.id,
    um.id AS user_mention_id,
    ac.creator_id,
    ac.creator_actor_id,
    ac.creator_local,
    ac.post_id,
    ac.post_name,
    ac.parent_id,
    ac.content,
    ac.removed,
    um.read,
    ac.published,
    ac.updated,
    ac.deleted,
    ac.community_id,
    ac.community_actor_id,
    ac.community_local,
    ac.community_name,
    ac.community_icon,
    ac.banned,
    ac.banned_from_community,
    ac.creator_name,
    ac.creator_preferred_username,
    ac.creator_avatar,
    ac.score,
    ac.upvotes,
    ac.downvotes,
    ac.hot_rank,
    ac.hot_rank_active,
    u.id AS user_id,
    COALESCE(cl.score::integer, 0) AS my_vote,
    ( SELECT cs.id::boolean AS id
           FROM comment_saved cs
          WHERE u.id = cs.user_id AND cs.comment_id = ac.id) AS saved,
    um.recipient_id,
    ( SELECT u_1.actor_id
           FROM user_ u_1
          WHERE u_1.id = um.recipient_id) AS recipient_actor_id,
    ( SELECT u_1.local
           FROM user_ u_1
          WHERE u_1.id = um.recipient_id) AS recipient_local
   FROM user_ u
     CROSS JOIN ( SELECT ca.id,
            ca.creator_id,
            ca.post_id,
            ca.parent_id,
            ca.content,
            ca.removed,
            ca.read,
            ca.published,
            ca.updated,
            ca.deleted,
            ca.ap_id,
            ca.local,
            ca.post_name,
            ca.community_id,
            ca.community_actor_id,
            ca.community_local,
            ca.community_name,
            ca.community_icon,
            ca.banned,
            ca.banned_from_community,
            ca.creator_actor_id,
            ca.creator_local,
            ca.creator_name,
            ca.creator_preferred_username,
            ca.creator_published,
            ca.creator_avatar,
            ca.creator_tags,
            ca.creator_community_tags,
            ca.score,
            ca.upvotes,
            ca.downvotes,
            ca.hot_rank,
            ca.hot_rank_active
           FROM hexbear.comment_fast_view ca) ac
     LEFT JOIN comment_like cl ON u.id = cl.user_id AND ac.id = cl.comment_id
     LEFT JOIN user_mention um ON um.comment_id = ac.id
UNION ALL
 SELECT ac.id,
    um.id AS user_mention_id,
    ac.creator_id,
    ac.creator_actor_id,
    ac.creator_local,
    ac.post_id,
    ac.post_name,
    ac.parent_id,
    ac.content,
    ac.removed,
    um.read,
    ac.published,
    ac.updated,
    ac.deleted,
    ac.community_id,
    ac.community_actor_id,
    ac.community_local,
    ac.community_name,
    ac.community_icon,
    ac.banned,
    ac.banned_from_community,
    ac.creator_name,
    ac.creator_preferred_username,
    ac.creator_avatar,
    ac.score,
    ac.upvotes,
    ac.downvotes,
    ac.hot_rank,
    ac.hot_rank_active,
    NULL::integer AS user_id,
    NULL::integer AS my_vote,
    NULL::boolean AS saved,
    um.recipient_id,
    ( SELECT u.actor_id
           FROM user_ u
          WHERE u.id = um.recipient_id) AS recipient_actor_id,
    ( SELECT u.local
           FROM user_ u
          WHERE u.id = um.recipient_id) AS recipient_local
   FROM hexbear.comment_fast_view ac
     LEFT JOIN user_mention um ON um.comment_id = ac.id;