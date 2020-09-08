create schema chapo;

-- Replace the comment_aggregate table with a 1:1 table solely to house aggregate values
create table chapo.comment_stat
(
    comment_id  int references public.comment on update cascade on delete cascade primary key,
    score int not null default 0,
    upvotes int not null default 0,
    downvotes int not null default 0,
    hot_rank int,
    hot_rank_active int
);

-- Create new trigger functions under 'chapo' for altered functionality
create or replace function chapo.refresh_comment_like()
    RETURNS trigger
    LANGUAGE 'plpgsql'
AS $BODY$
begin
  IF (TG_OP = 'DELETE') THEN
    update chapo.comment_stat
    set score = score - OLD.score,
    upvotes = case 
      when (OLD.score = 1) then upvotes - 1 
      else upvotes end,
    downvotes = case 
      when (OLD.score = -1) then downvotes - 1 
      else downvotes end
    where comment_id = OLD.comment_id;

  ELSIF (TG_OP = 'INSERT') THEN
    update chapo.comment_stat
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

create or replace function chapo.refresh_comment()
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

    update chapo.comment_stat
    set
      hot_rank = hot_rank(coalesce(score, 1)::numeric, (select published from post where id = NEW.post_id)),
      hot_rank_active = hot_rank(coalesce(score, 1)::numeric, NEW.published)
    where comment_id = NEW.id;

  ELSIF (TG_OP = 'INSERT') THEN

    insert into chapo.comment_stat (comment_id, hot_rank, hot_rank_active)
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

create or replace function chapo.refresh_user()
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

create or replace function chapo.refresh_community_user_ban()
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
drop trigger if exists refresh_user on public.user;
drop trigger if exists refresh_community_user_ban on public.community_user_ban;

-- Migrate stats (warning: could take time pending instance size, consider downtime)
insert into chapo.comment_stat (comment_id, score, upvotes, downvotes, hot_rank, hot_rank_active)
select id, score, upvotes, downvotes, hot_rank, hot_rank_active from comment_aggregates_fast;

-- Add new triggers
create trigger refresh_comment
    after insert or delete or update
    on public.comment
    for each row
    execute procedure chapo.refresh_comment();
  
create trigger refresh_comment_like
    after insert or delete
    on public.comment_like
    for each row
    execute procedure chapo.refresh_comment_like();

create trigger refresh_user
    after insert or delete or update
    on public.user
    for each row
    execute procedure chapo.refresh_user();

create trigger refresh_community_user_ban
    after insert or delete
    on public.community_user_ban
    for each row
    execute procedure chapo.refresh_community_user_ban();

-- Add altered view, this one combines the "aggregate view" into the normal view
create or replace view chapo.comment_fast_view as

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
	LEFT JOIN chapo.comment_stat ccs ON ccs.comment_id = ct.id
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
	LEFT JOIN chapo.comment_stat ccs ON ccs.comment_id = ct.id;