drop view hexbear.reply_fast_view;
create or replace view hexbear.reply_fast_view as select
	ct.id,
	ct.creator_id,
	ct.post_id,
	ct.parent_id,
	ct.content,
	ct.removed,
	replies.read,
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
	COALESCE(ccs.hot_rank, 0) AS hot_rank,
	COALESCE(ccs.hot_rank_active, 0) AS hot_rank_active,
  -- this is to preserve the api code
	me.id AS user_id,
	me.id as recipient_id,
	COALESCE(cl.score::integer, 0) AS my_vote,
	COALESCE(cf.id, 0)::boolean AS subscribed,
	COALESCE(cs.id, 0)::boolean AS saved
from user_ me
join lateral (
	select 
		c.id,
		p.creator_id, c.read
	from post p
	join comment c on c.post_id = p.id
	where p.creator_id = me.id 
	and c.creator_id <> p.creator_id
	and c.deleted is false and c.removed is false
	and p.deleted is false and p.removed is false
	and c.parent_id is null
union
	select 
		c2.id, c.creator_id, c2.read
	from comment c
	join comment c2 on c.id = c2.parent_id
	where c.creator_id = me.id and c.creator_id <> c2.creator_id
	and c.deleted is false and c.removed is false
	and c2.deleted is false and c2.removed is false
) replies on replies.creator_id = me.id
join comment ct on replies.id = ct.id
LEFT JOIN post p ON ct.post_id = p.id
LEFT JOIN community c ON p.community_id = c.id
LEFT JOIN user_ u ON ct.creator_id = u.id
LEFT JOIN user_tag ut ON ct.creator_id = ut.user_id
LEFT JOIN community_user_tag cut ON ct.creator_id = cut.user_id AND p.community_id = cut.community_id
LEFT JOIN community_user_ban cb ON ct.creator_id = cb.user_id AND p.id = ct.post_id AND p.community_id = cb.community_id
LEFT JOIN hexbear.comment_stat ccs ON ccs.comment_id = ct.id
LEFT JOIN comment_like cl ON me.id = cl.user_id AND cl.comment_id = ct.id
LEFT JOIN comment_saved cs ON me.id = cs.user_id AND cs.comment_id = ct.id
LEFT JOIN community_follower cf ON me.id = cf.user_id AND p.community_id = cf.community_id;

drop view hexbear.user_mention_fast_view;
CREATE OR REPLACE VIEW hexbear.user_mention_fast_view
 AS
 SELECT 
 	ct.id,
	ct.creator_id,
	ct.post_id,
	ct.parent_id,
	ct.content,
	ct.removed,
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
	COALESCE(ccs.hot_rank, 0) AS hot_rank,
	COALESCE(ccs.hot_rank_active, 0) AS hot_rank_active,
	COALESCE(cl.score::integer, 0) AS my_vote,
	COALESCE(cf.id, 0)::boolean AS subscribed,
	COALESCE(cs.id, 0)::boolean AS saved,
  um.id AS user_mention_id,
  um.read,
  um.recipient_id,
  um.recipient_id as user_id,
  me.actor_id AS recipient_actor_id,
  me.local AS recipient_local
  FROM user_mention um
  join user_ me on um.recipient_id = me.id
  join comment ct on um.comment_id = ct.id
	LEFT JOIN post p ON ct.post_id = p.id
	LEFT JOIN community c ON p.community_id = c.id
	LEFT JOIN user_ u ON ct.creator_id = u.id
	LEFT JOIN user_tag ut ON ct.creator_id = ut.user_id
	LEFT JOIN community_user_tag cut ON ct.creator_id = cut.user_id AND p.community_id = cut.community_id
	LEFT JOIN community_user_ban cb ON ct.creator_id = cb.user_id AND p.id = ct.post_id AND p.community_id = cb.community_id
	LEFT JOIN hexbear.comment_stat ccs ON ccs.comment_id = ct.id
	LEFT JOIN comment_like cl ON me.id = cl.user_id AND cl.comment_id = ct.id
	LEFT JOIN comment_saved cs ON me.id = cs.user_id AND cs.comment_id = ct.id
	LEFT JOIN community_follower cf ON me.id = cf.user_id AND p.community_id = cf.community_id;

-- No perf gain just cutting down verbosity
CREATE OR REPLACE VIEW public.mod_add_community_view
 AS
 SELECT ma.id,
    ma.mod_user_id,
    ma.other_user_id,
    ma.community_id,
    ma.removed,
    ma.when_,
    mod_user.name AS mod_user_name,
    u.name AS other_user_name,
    c.name AS community_name
   FROM mod_add_community ma
   join user_ mod_user on mod_user.id = ma.mod_user_id
   join user_ u on u.id = ma.other_user_id
   join community c on c.id = ma.community_id;

CREATE OR REPLACE VIEW public.mod_add_view
 AS
 SELECT ma.id,
    ma.mod_user_id,
    ma.other_user_id,
    ma.removed,
    ma.when_,
    mod_user.name AS mod_user_name,
    u.name AS other_user_name
   FROM mod_add ma
   join user_ mod_user on mod_user.id = ma.mod_user_id
   join user_ u on u.id = ma.other_user_id;

CREATE OR REPLACE VIEW public.mod_ban_from_community_view
 AS
 SELECT mb.id,
    mb.mod_user_id,
    mb.other_user_id,
    mb.community_id,
    mb.reason,
    mb.banned,
    mb.expires,
    mb.when_,
    mod_user.name AS mod_user_name,
    u.name AS other_user_name,
    c.name AS community_name
   from mod_ban_from_community mb
   join user_ mod_user on mod_user.id = mb.mod_user_id
   join user_ u on u.id = mb.other_user_id
   join community c on c.id = mb.community_id;

CREATE OR REPLACE VIEW public.mod_ban_view
 AS
 SELECT mb.id,
    mb.mod_user_id,
    mb.other_user_id,
    mb.reason,
    mb.banned,
    mb.expires,
    mb.when_,
    mod_user.name AS mod_user_name,
    u.name AS other_user_name
   from mod_ban mb
   join user_ mod_user on mod_user.id = mb.mod_user_id
   join user_ u on u.id = mb.other_user_id;

-- Very mild improvement for not re-querying posts
CREATE OR REPLACE VIEW public.mod_lock_post_view
 AS
 SELECT mlp.id,
    mlp.mod_user_id,
    mlp.post_id,
    mlp.locked,
    mlp.when_,
    u.name AS mod_user_name,
    p.name AS post_name,
    c.id AS community_id,
    c.name AS community_name
   FROM mod_lock_post mlp
   join user_ u on u.id = mlp.mod_user_id
   join post p on p.id = mlp.post_id
   join community c on c.id = p.community_id;

-- 23k cost to ~500
CREATE OR REPLACE VIEW public.mod_remove_comment_view
 AS
 SELECT mrc.id,
    mrc.mod_user_id,
    mrc.comment_id,
    mrc.reason,
    mrc.removed,
    mrc.when_,
    mod_user.name AS mod_user_name,
    commentator.id AS comment_user_id,
   	commentator.name AS comment_user_name,
    c.content AS comment_content,
   	p.id AS post_id,
    p.name AS post_name,
    co.id AS community_id,
   	co.name AS community_name
   FROM mod_remove_comment mrc
   join user_ mod_user on mod_user.id = mrc.mod_user_id
   join comment c on c.id = mrc.comment_id
   join post p on p.id = c.post_id
   join community co on co.id = p.community_id
   join user_ commentator on commentator.id = c.creator_id;

CREATE OR REPLACE VIEW public.mod_remove_community_view
 AS
 SELECT mrc.id,
    mrc.mod_user_id,
    mrc.community_id,
    mrc.reason,
    mrc.removed,
    mrc.expires,
    mrc.when_,
    u.name AS mod_user_name,
    c.name AS community_name
   FROM mod_remove_community mrc
   join user_ u on u.id = mrc.mod_user_id
   join community c on c.id = mrc.community_id;

CREATE OR REPLACE VIEW public.mod_remove_post_view
 AS
 SELECT mrp.id,
    mrp.mod_user_id,
    mrp.post_id,
    mrp.reason,
    mrp.removed,
    mrp.when_,
    u.name AS mod_user_name,
    p.name AS post_name,
    c.id AS community_id,
    c.name AS community_name
   FROM mod_remove_post mrp
   join user_ u on u.id = mrp.mod_user_id
   join post p on p.id = mrp.post_id
   join community c on c.id = p.community_id;

CREATE OR REPLACE VIEW public.mod_sticky_post_view
 AS
 SELECT msp.id,
    msp.mod_user_id,
    msp.post_id,
    msp.stickied,
    msp.when_,
    u.name AS mod_user_name,
    p.name AS post_name,
    c.id AS community_id,
    c.name AS community_name
   FROM mod_sticky_post msp
   join user_ u on u.id = msp.mod_user_id
   join post p on p.id = msp.post_id
   join community c on c.id = p.community_id;

-- This provides an estimate of each count to avoid a full table scan, as long as regular autovaccum/analyzing happens this will be dead on to ~95% accurate
-- and we want autovac/analyzes to be happening frequently 
CREATE OR REPLACE VIEW public.site_view
 AS
 SELECT s.id,
    s.name,
    s.description,
    s.creator_id,
    s.published,
    s.updated,
    s.enable_downvotes,
    s.open_registration,
    s.enable_nsfw,
    s.enable_create_communities,
    s.icon,
    s.banner,
    u.name AS creator_name,
    u.preferred_username AS creator_preferred_username,
    u.avatar AS creator_avatar,
    ( SELECT (reltuples/NULLIF(relpages, 0)) * (
    	   pg_relation_size('user_') /
    	    NULLIF((current_setting('block_size')::integer), 0))
		FROM pg_class where relname = 'user_'
	)::bigint AS number_of_users,
    ( SELECT (reltuples/NULLIF(relpages, 0)) * (
   	   pg_relation_size('post') /
		    NULLIF((current_setting('block_size')::integer), 0))
	 	FROM pg_class where relname = 'post'
	)::bigint AS number_of_posts,
    ( SELECT (reltuples/NULLIF(relpages, 0)) * (
		   pg_relation_size('comment') /
    	    NULLIF((current_setting('block_size')::integer), 0))
	 	FROM pg_class where relname = 'comment'
	)::bigint AS number_of_comments,
    ( SELECT (reltuples/NULLIF(relpages, 0)) * (
    	   pg_relation_size('community') /
   	    NULLIF((current_setting('block_size')::integer), 0))
		FROM pg_class where relname = 'community'
	)::bigint AS number_of_communities
   FROM site s
     LEFT JOIN user_ u ON s.creator_id = u.id;