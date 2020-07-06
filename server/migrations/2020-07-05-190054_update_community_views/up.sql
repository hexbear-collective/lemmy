DROP VIEW community_moderator_view;
DROP VIEW community_follower_view;
DROP VIEW community_user_ban_view;
DROP VIEW community_mview;
DROP VIEW community_view;
DROP MATERIALIZED VIEW community_aggregates_mview;
DROP VIEW community_aggregates_view;

CREATE VIEW community_aggregates_view AS
SELECT 
    c.id,
    c.name,
    c.title,
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
    u.avatar AS creator_avatar,
    cat.name AS category_name,
    COALESCE(cf.subs, 0) AS number_of_subscribers,
    COALESCE(cd.posts, 0) AS number_of_posts,
    COALESCE(cd.COMMENTS, 0) AS number_of_comments,
    hot_rank(cf.subs, c.published) AS hot_rank
FROM community c
LEFT JOIN user_ u ON c.creator_id = u.id
LEFT JOIN category cat ON c.category_id = cat.id
LEFT JOIN (
    SELECT
        p.community_id,
        COUNT(DISTINCT p.id) AS posts,
        COUNT(DISTINCT ct.id) AS comments
    FROM post p
    JOIN comment ct ON p.id = ct.post_id
    GROUP BY p.community_id
) cd ON cd.community_id = c.id
LEFT JOIN (
    SELECT
        community_id,
        COUNT(*) AS subs 
    FROM community_follower
    GROUP BY community_id 
) cf ON cf.community_id = c.id;

CREATE MATERIALIZED VIEW community_aggregates_mview AS SELECT * FROM community_aggregates_view;
CREATE UNIQUE INDEX idx_community_aggregates_mview_id ON community_aggregates_mview (id);

CREATE VIEW community_view AS
SELECT
    cv.*,
    us.user AS user_id,
    us.is_subbed::bool AS subscribed
FROM community_aggregates_view cv
CROSS JOIN LATERAL (
	SELECT
		u.id AS user,
		COALESCE(cf.community_id, 0) AS is_subbed
	FROM user_ u
	LEFT JOIN community_follower cf ON u.id = cf.user_id AND cf.community_id = cv.id
) AS us

UNION ALL

SELECT 
    cv.*,
    null AS user_id,
    null AS subscribed
FROM community_aggregates_view cv;

CREATE VIEW community_mview AS SELECT * FROM community_view;

CREATE VIEW community_moderator_view AS
SELECT
    cm.*,
    u.actor_id AS user_actor_id,
    u.local AS user_local,
    u.name AS user_name,
    u.avatar AS avatar,
    c.actor_id AS community_actor_id,
    c.local AS community_local,
    c.name AS community_name
FROM community_moderator cm
LEFT JOIN user_ u on cm.user_id = u.id
LEFT JOIN community c ON cm.community_id = c.id;

CREATE VIEW community_follower_view AS
SELECT
    cf.*,
    u.actor_id AS user_actor_id,
    u.local AS user_local,
    u.name AS user_name,
    u.avatar AS avatar,
    c.actor_id AS community_actor_id,
    c.local AS community_local,
    c.name AS community_name
FROM community_follower cf
LEFT JOIN user_ u on cf.user_id = u.id
LEFT JOIN community c ON cf.community_id = c.id;

CREATE VIEW community_user_ban_view AS
SELECT
    cb.*,
    u.actor_id AS user_actor_id,
    u.local AS user_local,
    u.name AS user_name,
    u.avatar AS avatar,
    c.actor_id AS community_actor_id,
    c.local AS community_local,
    c.name AS community_name
FROM community_user_ban cb
LEFT JOIN user_ u on cb.user_id = u.id
LEFT JOIN community c ON cb.community_id = c.id;
