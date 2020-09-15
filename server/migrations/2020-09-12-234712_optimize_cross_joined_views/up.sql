create index if not exists "idx_community_follower_user_id" on public.community_follower (user_id);
create index if not exists "idx_comment_saved_user_id" on public.comment_saved (user_id);
-- Before: 918k cost, 450ms exec time
-- After: 6k cost, 143ms exec (user id filter only)

CREATE OR REPLACE VIEW hexbear.post_fast_view
 AS
 SELECT pav.id,
    pav.name,
    pav.url,
    pav.body,
    pav.creator_id,
    pav.community_id,
    pav.removed,
    pav.locked,
    pav.published,
    pav.updated,
    pav.deleted,
    pav.nsfw,
    pav.stickied,
    pav.embed_title,
    pav.embed_description,
    pav.embed_html,
    pav.thumbnail_url,
    pav.ap_id,
    pav.local,
    pav.creator_actor_id,
    pav.creator_local,
    pav.creator_name,
    pav.creator_preferred_username,
    pav.creator_published,
    pav.creator_avatar,
    pav.creator_tags,
    pav.creator_community_tags,
    pav.banned,
    pav.banned_from_community,
    pav.community_actor_id,
    pav.community_local,
    pav.community_name,
    pav.community_icon,
    pav.community_removed,
    pav.community_deleted,
    pav.community_nsfw,
    pav.number_of_comments,
    pav.score,
    pav.upvotes,
    pav.downvotes,
    pav.hot_rank,
    pav.hot_rank_active,
    pav.newest_activity_time,
    pav.user_id,
    pav.my_vote,
    pav.subscribed::boolean AS subscribed,
    pav.read::boolean AS read,
    pav.saved::boolean AS saved
   FROM ( SELECT p.id,
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
            me.id AS user_id,
            COALESCE(cf.community_id, 0) AS subscribed,
            COALESCE(pr.post_id, 0) AS read,
            COALESCE(ps.post_id, 0) AS saved,
            COALESCE(pl.score::integer, 0) AS my_vote
           FROM post p
             LEFT JOIN user_ u ON p.creator_id = u.id
             LEFT JOIN user_tag ut ON p.creator_id = ut.user_id
             LEFT JOIN community_user_tag cut ON p.creator_id = cut.user_id AND p.community_id = cut.community_id
             LEFT JOIN community_user_ban cb ON p.creator_id = cb.user_id AND p.community_id = cb.community_id
             LEFT JOIN community c ON p.community_id = c.id
             LEFT JOIN hexbear.post_stat cps ON cps.post_id = p.id
             CROSS JOIN user_ me
             LEFT JOIN community_follower cf ON me.id = cf.user_id AND cf.community_id = p.community_id
             LEFT JOIN post_read pr ON me.id = pr.user_id AND pr.post_id = p.id
             LEFT JOIN post_saved ps ON me.id = ps.user_id AND ps.post_id = p.id
             LEFT JOIN post_like pl ON me.id = pl.user_id AND p.id = pl.post_id) pav
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
    NULL::integer AS user_id,
    NULL::integer AS my_vote,
    NULL::boolean AS subscribed,
    NULL::boolean AS read,
    NULL::boolean AS saved
   FROM post p
     LEFT JOIN user_ u ON p.creator_id = u.id
     LEFT JOIN user_tag ut ON p.creator_id = ut.user_id
     LEFT JOIN community_user_tag cut ON p.creator_id = cut.user_id AND p.community_id = cut.community_id
     LEFT JOIN community_user_ban cb ON p.creator_id = cb.user_id AND p.community_id = cb.community_id
     LEFT JOIN community c ON p.community_id = c.id
     LEFT JOIN hexbear.post_stat cps ON cps.post_id = p.id;

-- Before 5.8mil, 2250ms / 1k and 1ms wth a post_id
-- After: 31k cost, 945ms (user id only filter) / 800 and 1ms with a post_id
CREATE OR REPLACE VIEW hexbear.comment_fast_view
 AS
 SELECT cav.id,
    cav.creator_id,
    cav.post_id,
    cav.parent_id,
    cav.content,
    cav.removed,
    cav.read,
    cav.published,
    cav.updated,
    cav.deleted,
    cav.ap_id,
    cav.local,
    cav.post_name,
    cav.community_id,
    cav.community_actor_id,
    cav.community_local,
    cav.community_name,
    cav.community_icon,
    cav.banned,
    cav.banned_from_community,
    cav.creator_actor_id,
    cav.creator_local,
    cav.creator_name,
    cav.creator_preferred_username,
    cav.creator_published,
    cav.creator_avatar,
    cav.creator_tags,
    cav.creator_community_tags,
    cav.score,
    cav.upvotes,
    cav.downvotes,
    cav.hot_rank,
    cav.hot_rank_active,
    cav.user_id,
    cav.my_vote,
    cav.subscribed,
    cav.saved
   FROM ( SELECT ct.id,
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
            COALESCE(ccs.hot_rank, 0) AS hot_rank,
            COALESCE(ccs.hot_rank_active, 0) AS hot_rank_active,
		 	me.id as user_id,
		 	COALESCE(cl.score::integer, 0) AS my_vote,
            COALESCE(cf.id, 0)::boolean AS subscribed,
            COALESCE(cs.id, 0)::boolean AS saved
           FROM comment ct
             LEFT JOIN post p ON ct.post_id = p.id
             LEFT JOIN community c ON p.community_id = c.id
             LEFT JOIN user_ u ON ct.creator_id = u.id
             LEFT JOIN user_tag ut ON ct.creator_id = ut.user_id
             LEFT JOIN community_user_tag cut ON ct.creator_id = cut.user_id AND p.community_id = cut.community_id
             LEFT JOIN community_user_ban cb ON ct.creator_id = cb.user_id AND p.id = ct.post_id AND p.community_id = cb.community_id
             LEFT JOIN hexbear.comment_stat ccs ON ccs.comment_id = ct.id
		 	cross join user_ me
			 LEFT JOIN comment_like cl ON me.id = cl.user_id AND cl.comment_id = ct.id
             LEFT JOIN comment_saved cs ON me.id = cs.user_id AND cs.comment_id = ct.id
             LEFT JOIN community_follower cf ON me.id = cf.user_id AND p.community_id = cf.community_id ) cav
UNION ALL
 SELECT ct.id,
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
    COALESCE(ccs.hot_rank, 0) AS hot_rank,
    COALESCE(ccs.hot_rank_active, 0) AS hot_rank_active,
    NULL::integer AS user_id,
    NULL::integer AS my_vote,
    NULL::boolean AS subscribed,
    NULL::boolean AS saved
   FROM comment ct
     LEFT JOIN post p ON ct.post_id = p.id
     LEFT JOIN community c ON p.community_id = c.id
     LEFT JOIN user_ u ON ct.creator_id = u.id
     LEFT JOIN user_tag ut ON ct.creator_id = ut.user_id
     LEFT JOIN community_user_tag cut ON ct.creator_id = cut.user_id AND p.community_id = cut.community_id
     LEFT JOIN community_user_ban cb ON ct.creator_id = cb.user_id AND p.id = ct.post_id AND p.community_id = cb.community_id
     LEFT JOIN hexbear.comment_stat ccs ON ccs.comment_id = ct.id;

-- This one isnt bad, a minor cost reduction but mostly because the community table isnt big
CREATE OR REPLACE VIEW hexbear.community_fast_view
 AS
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
    me.id::integer AS user_id,
    cf.id::boolean AS subscribed
   FROM community c
     LEFT JOIN user_ u ON c.creator_id = u.id
     LEFT JOIN category cat ON c.category_id = cat.id
     LEFT JOIN hexbear.community_stat ccs ON ccs.community_id = c.id
	 cross join user_ me
	 left join community_follower cf on cf.user_id = me.id and cf.community_id = c.id
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
     LEFT JOIN hexbear.community_stat ccs ON ccs.community_id = c.id;

-- Before: 1.6bil cost, ???? (dont run this on prod)
-- After: should equal comment fast view at 20-30k and ~100ms
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
    ac.user_id AS user_id,
    ac.my_vote AS my_vote,
	ac.saved,
	um.recipient_id,
	u.actor_id as recipient_actor_id,
	u.local as recipient_local
   FROM user_mention um
     LEFT JOIN hexbear.comment_fast_view ac ON um.comment_id = ac.id
	 left join user_ u on u.id = um.recipient_id
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
	u.actor_id as recipient_actor_id,
	u.local as recipient_local
   FROM user_mention um
     LEFT JOIN hexbear.comment_fast_view ac ON um.comment_id = ac.id
	 left join user_ u on u.id = um.recipient_id