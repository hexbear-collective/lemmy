-- Replace the pommunity_aggregate table with a 1:1 table solely to house aggregate values
create table hexbear.community_stat
(
    community_id int references public.community on update cascade on delete cascade primary key,
    number_of_subscribers int not null default 0,
    number_of_posts int not null default 0,
    number_of_comments int not null default 0,
    hot_rank int
);

-- Create new trigger functions under 'hexbear' for altered functionality
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

create or replace function hexbear.refresh_community_follower()
    RETURNS trigger
    LANGUAGE 'plpgsql'
AS $BODY$
begin
  IF (TG_OP = 'DELETE') THEN
    update hexbear.community_stat set number_of_subscribers = number_of_subscribers - 1 where community_id = OLD.community_id;
  ELSIF (TG_OP = 'INSERT') THEN
    update hexbear.community_stat set number_of_subscribers = number_of_subscribers + 1 where community_id = NEW.community_id;
  END IF;

  return null;
end $BODY$;

drop trigger if exists refresh_post ON public.post;
drop trigger if exists refresh_comment on public.comment;
drop trigger if exists refresh_community on public.community;
drop trigger if exists refresh_community_follower on public.community_follower;

-- Migrate stats (warning: could take time pending instance size, consider downtime)
insert into hexbear.community_stat (community_id, number_of_subscribers, number_of_posts, number_of_comments, hot_rank)
select id, number_of_subscribers, number_of_posts, number_of_comments, hot_rank from community_aggregates_fast;

-- Add new triggers
create trigger refresh_comment
    after insert or delete or update
    on public.comment
    for each row
    execute procedure hexbear.refresh_comment();
  
create trigger refresh_post
    after insert or delete or update
    on public.post
    for each row
    execute procedure hexbear.refresh_post();

create trigger refresh_community
    after insert or update
    on public.community
    for each row
    execute procedure hexbear.refresh_community();

create trigger refresh_community_follower
    after insert or delete
    on public.community_follower
    for each row
    execute procedure hexbear.refresh_community_follower();

CREATE OR REPLACE VIEW hexbear.community_fast_view
 AS
 SELECT ac.id,
    ac.name,
    ac.title,
    ac.icon,
    ac.banner,
    ac.description,
    ac.category_id,
    ac.creator_id,
    ac.removed,
    ac.published,
    ac.updated,
    ac.deleted,
    ac.nsfw,
    ac.actor_id,
    ac.local,
    ac.last_refreshed_at,
    ac.creator_actor_id,
    ac.creator_local,
    ac.creator_name,
    ac.creator_preferred_username,
    ac.creator_avatar,
    ac.category_name,
    ac.number_of_subscribers,
    ac.number_of_posts,
    ac.number_of_comments,
    ac.hot_rank,
    u.id AS user_id,
    ( SELECT cf.id::boolean AS id
           FROM community_follower cf
          WHERE u.id = cf.user_id AND ac.id = cf.community_id) AS subscribed
   FROM user_ u
     CROSS JOIN (
		 SELECT c.id,
		c.name,
		c.title,
		c.icon,
		c.banner,
		c.description,
		c.category_id,
		c.creator_id,
		c.removed,
		c.published,
		c.updated,
		c.deleted,
		c.nsfw,
		c.actor_id,
		c.local,
		c.last_refreshed_at,
		u.actor_id AS creator_actor_id,
		u.local AS creator_local,
		u.name AS creator_name,
		u.preferred_username AS creator_preferred_username,
		u.avatar AS creator_avatar,
		cat.name AS category_name,
		COALESCE(ccs.number_of_subscribers, 0)::bigint AS number_of_subscribers,
		COALESCE(ccs.number_of_posts, 0)::bigint AS number_of_posts,
		COALESCE(ccs.number_of_comments, 0)::bigint AS number_of_comments,
		COALESCE(ccs.hot_rank, 0) AS hot_rank
	   FROM community c
		 LEFT JOIN user_ u ON c.creator_id = u.id
		 LEFT JOIN category cat ON c.category_id = cat.id
		 LEFT JOIN hexbear.community_stat ccs on ccs.community_id = c.id
	) ac
	 
UNION ALL
 SELECT c.id,
    c.name,
    c.title,
    c.icon,
    c.banner,
    c.description,
    c.category_id,
    c.creator_id,
    c.removed,
    c.published,
    c.updated,
    c.deleted,
    c.nsfw,
    c.actor_id,
    c.local,
    c.last_refreshed_at,
    u.actor_id AS creator_actor_id,
    u.local AS creator_local,
    u.name AS creator_name,
    u.preferred_username AS creator_preferred_username,
    u.avatar AS creator_avatar,
    cat.name AS category_name,
    COALESCE(ccs.number_of_subscribers, 0)::bigint AS number_of_subscribers,
    COALESCE(ccs.number_of_posts, 0)::bigint AS number_of_posts,
    COALESCE(ccs.number_of_comments, 0)::bigint AS number_of_comments,
   	COALESCE(ccs.hot_rank, 0) AS hot_rank,
	  NULL::integer AS user_id,
    NULL::boolean AS subscribed
  FROM community c
    LEFT JOIN user_ u ON c.creator_id = u.id
    LEFT JOIN category cat ON c.category_id = cat.id
    LEFT JOIN hexbear.community_stat ccs on ccs.community_id = c.id;
