alter table post add column featured boolean default false not null;

drop view hexbear.post_fast_view;

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
    pav.featured,
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
            p.featured,
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
    p.featured,
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
