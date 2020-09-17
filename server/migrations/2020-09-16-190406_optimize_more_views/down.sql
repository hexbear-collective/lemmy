drop view hexbear.reply_fast_view;
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

drop view hexbear.user_mention_fast_view;
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
    ac.user_id,
    ac.my_vote,
    ac.saved,
    um.recipient_id,
    u.actor_id AS recipient_actor_id,
    u.local AS recipient_local
   FROM user_mention um
     LEFT JOIN hexbear.comment_fast_view ac ON um.comment_id = ac.id
     LEFT JOIN user_ u ON u.id = um.recipient_id
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
    u.actor_id AS recipient_actor_id,
    u.local AS recipient_local
   FROM user_mention um
     LEFT JOIN hexbear.comment_fast_view ac ON um.comment_id = ac.id
     LEFT JOIN user_ u ON u.id = um.recipient_id;

CREATE OR REPLACE VIEW public.mod_add_community_view
 AS
 SELECT ma.id,
    ma.mod_user_id,
    ma.other_user_id,
    ma.community_id,
    ma.removed,
    ma.when_,
    ( SELECT u.name
           FROM user_ u
          WHERE ma.mod_user_id = u.id) AS mod_user_name,
    ( SELECT u.name
           FROM user_ u
          WHERE ma.other_user_id = u.id) AS other_user_name,
    ( SELECT c.name
           FROM community c
          WHERE ma.community_id = c.id) AS community_name
   FROM mod_add_community ma;
   
CREATE OR REPLACE VIEW public.mod_add_view
 AS
 SELECT ma.id,
    ma.mod_user_id,
    ma.other_user_id,
    ma.removed,
    ma.when_,
    ( SELECT u.name
           FROM user_ u
          WHERE ma.mod_user_id = u.id) AS mod_user_name,
    ( SELECT u.name
           FROM user_ u
          WHERE ma.other_user_id = u.id) AS other_user_name
   FROM mod_add ma;

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
    ( SELECT u.name
           FROM user_ u
          WHERE mb.mod_user_id = u.id) AS mod_user_name,
    ( SELECT u.name
           FROM user_ u
          WHERE mb.other_user_id = u.id) AS other_user_name,
    ( SELECT c.name
           FROM community c
          WHERE mb.community_id = c.id) AS community_name
   FROM mod_ban_from_community mb;

CREATE OR REPLACE VIEW public.mod_ban_view
 AS
 SELECT mb.id,
    mb.mod_user_id,
    mb.other_user_id,
    mb.reason,
    mb.banned,
    mb.expires,
    mb.when_,
    ( SELECT u.name
           FROM user_ u
          WHERE mb.mod_user_id = u.id) AS mod_user_name,
    ( SELECT u.name
           FROM user_ u
          WHERE mb.other_user_id = u.id) AS other_user_name
   FROM mod_ban mb;

CREATE OR REPLACE VIEW public.mod_lock_post_view
 AS
 SELECT mlp.id,
    mlp.mod_user_id,
    mlp.post_id,
    mlp.locked,
    mlp.when_,
    ( SELECT u.name
           FROM user_ u
          WHERE mlp.mod_user_id = u.id) AS mod_user_name,
    ( SELECT p.name
           FROM post p
          WHERE mlp.post_id = p.id) AS post_name,
    ( SELECT c.id
           FROM post p,
            community c
          WHERE mlp.post_id = p.id AND p.community_id = c.id) AS community_id,
    ( SELECT c.name
           FROM post p,
            community c
          WHERE mlp.post_id = p.id AND p.community_id = c.id) AS community_name
   FROM mod_lock_post mlp;

CREATE OR REPLACE VIEW public.mod_remove_comment_view
 AS
 SELECT mrc.id,
    mrc.mod_user_id,
    mrc.comment_id,
    mrc.reason,
    mrc.removed,
    mrc.when_,
    ( SELECT u.name
           FROM user_ u
          WHERE mrc.mod_user_id = u.id) AS mod_user_name,
    ( SELECT c.id
           FROM comment c
          WHERE mrc.comment_id = c.id) AS comment_user_id,
    ( SELECT u.name
           FROM user_ u,
            comment c
          WHERE mrc.comment_id = c.id AND u.id = c.creator_id) AS comment_user_name,
    ( SELECT c.content
           FROM comment c
          WHERE mrc.comment_id = c.id) AS comment_content,
    ( SELECT p.id
           FROM post p,
            comment c
          WHERE mrc.comment_id = c.id AND c.post_id = p.id) AS post_id,
    ( SELECT p.name
           FROM post p,
            comment c
          WHERE mrc.comment_id = c.id AND c.post_id = p.id) AS post_name,
    ( SELECT co.id
           FROM comment c,
            post p,
            community co
          WHERE mrc.comment_id = c.id AND c.post_id = p.id AND p.community_id = co.id) AS community_id,
    ( SELECT co.name
           FROM comment c,
            post p,
            community co
          WHERE mrc.comment_id = c.id AND c.post_id = p.id AND p.community_id = co.id) AS community_name
   FROM mod_remove_comment mrc;

CREATE OR REPLACE VIEW public.mod_remove_community_view
 AS
 SELECT mrc.id,
    mrc.mod_user_id,
    mrc.community_id,
    mrc.reason,
    mrc.removed,
    mrc.expires,
    mrc.when_,
    ( SELECT u.name
           FROM user_ u
          WHERE mrc.mod_user_id = u.id) AS mod_user_name,
    ( SELECT c.name
           FROM community c
          WHERE mrc.community_id = c.id) AS community_name
   FROM mod_remove_community mrc;

CREATE OR REPLACE VIEW public.mod_remove_post_view
 AS
 SELECT mrp.id,
    mrp.mod_user_id,
    mrp.post_id,
    mrp.reason,
    mrp.removed,
    mrp.when_,
    ( SELECT u.name
           FROM user_ u
          WHERE mrp.mod_user_id = u.id) AS mod_user_name,
    ( SELECT p.name
           FROM post p
          WHERE mrp.post_id = p.id) AS post_name,
    ( SELECT c.id
           FROM post p,
            community c
          WHERE mrp.post_id = p.id AND p.community_id = c.id) AS community_id,
    ( SELECT c.name
           FROM post p,
            community c
          WHERE mrp.post_id = p.id AND p.community_id = c.id) AS community_name
   FROM mod_remove_post mrp;

CREATE OR REPLACE VIEW public.mod_sticky_post_view
 AS
 SELECT msp.id,
    msp.mod_user_id,
    msp.post_id,
    msp.stickied,
    msp.when_,
    ( SELECT u.name
           FROM user_ u
          WHERE msp.mod_user_id = u.id) AS mod_user_name,
    ( SELECT p.name
           FROM post p
          WHERE msp.post_id = p.id) AS post_name,
    ( SELECT c.id
           FROM post p,
            community c
          WHERE msp.post_id = p.id AND p.community_id = c.id) AS community_id,
    ( SELECT c.name
           FROM post p,
            community c
          WHERE msp.post_id = p.id AND p.community_id = c.id) AS community_name
   FROM mod_sticky_post msp;

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
    ( SELECT count(*) AS count
           FROM user_) AS number_of_users,
    ( SELECT count(*) AS count
           FROM post) AS number_of_posts,
    ( SELECT count(*) AS count
           FROM comment) AS number_of_comments,
    ( SELECT count(*) AS count
           FROM community) AS number_of_communities
   FROM site s
     LEFT JOIN user_ u ON s.creator_id = u.id;