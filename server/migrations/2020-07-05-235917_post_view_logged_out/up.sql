drop view post_mview;
drop view post_view;

CREATE OR REPLACE VIEW post_view AS
SELECT
    pav.*,
    us.id AS user_id,
    us.user_vote AS my_vote,
    us.is_banned::bool AS banned_from_community,
    us.is_subbed::bool AS subscribed,
    us.is_read::bool AS read,
    us.is_saved::bool AS saved
FROM post_aggregates_view pav
CROSS JOIN LATERAL (
    SELECT
        u.id,
        COALESCE(cb.id, 0) AS is_banned,
        COALESCE(cf.community_id, 0) AS is_subbed,
        COALESCE(pr.post_id, 0) AS is_read,
        COALESCE(ps.post_id, 0) AS is_saved,
        COALESCE(pl.score, 0) AS user_vote
    FROM user_ u
    LEFT JOIN community_user_ban cb ON u.id = cb.user_id AND cb.community_id = pav.community_id
    LEFT JOIN community_follower cf ON u.id = cf.user_id AND cf.community_id = pav.community_id
    LEFT JOIN post_read pr ON u.id = pr.user_id AND pr.post_id = pav.id
    LEFT JOIN post_saved ps ON u.id = ps.user_id AND ps.post_id = pav.id
    LEFT JOIN post_like pl ON u.id = pl.user_id AND pav.id = pl.post_id
) AS us

UNION ALL

SELECT 
pav.*,
null AS user_id,
null AS my_vote,
null AS banned_from_community,
null AS subscribed,
null AS read,
null AS saved
FROM post_aggregates_view pav;

CREATE OR REPLACE VIEW post_mview AS SELECT * FROM post_view;