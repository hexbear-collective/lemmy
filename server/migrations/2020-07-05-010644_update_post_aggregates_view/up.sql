DROP VIEW post_view;
DROP VIEW post_mview;
DROP MATERIALIZED VIEW post_aggregates_mview;
DROP VIEW post_aggregates_view;

CREATE VIEW post_aggregates_view AS
SELECT
	p.*,
	-- creator details
	u.actor_id AS creator_actor_id,
	u."name" AS creator_name,
	COALESCE(cb.community_id, 0) AS banned,
	u."local" AS creator_local,
	u.avatar AS creator_avatar,
	-- community details
	c.actor_id AS community_actor_id,
	c."local" AS community_local,
	c."name" AS community_name,
	c.removed AS community_removed,
	c.deleted AS community_deleted,
	c.nsfw AS community_nsfw,
	-- post score data/comment count
	COALESCE(ct.comments, 0) AS number_of_comments,
	COALESCE(pl.score, 0) AS score,
	COALESCE(pl.upvotes, 0) AS upvotes,
	COALESCE(pl.downvotes, 0) AS downvotes,
	hot_rank(
		COALESCE(pl.score , 0), (
			CASE
				WHEN (p.published < ('now'::timestamp - '1 month'::INTERVAL))
				THEN p.published
				ELSE GREATEST(ct.recent_comment_time, p.published)
			END
		)
	) AS hot_rank,
	(
		CASE
			WHEN (p.published < ('now'::timestamp - '1 month'::INTERVAL))
			THEN p.published
			ELSE GREATEST(ct.recent_comment_time, p.published)
		END
	) AS newest_activity_time
FROM post p
LEFT JOIN user_ u ON p.creator_id = u.id
LEFT JOIN community_user_ban cb ON p.creator_id = cb.user_id AND p.community_id = cb.community_id
LEFT JOIN community c ON p.community_id = c.id
LEFT JOIN (
	SELECT
		post_id,
		count(*) AS comments,
		MAX(published) AS recent_comment_time
	FROM comment
	GROUP BY post_id
) ct ON ct.post_id = p.id
LEFT JOIN (
	SELECT
		post_id,
		SUM(score) AS score,
		SUM(score) FILTER (WHERE score = 1) AS upvotes,
		SUM(score) FILTER (WHERE score = -1) AS downvotes
	FROM post_like
	GROUP BY post_id
) pl ON pl.post_id = p.id
ORDER BY p.id;

CREATE MATERIALIZED VIEW post_aggregates_mview AS SELECT * FROM post_aggregates_view;

CREATE UNIQUE INDEX idx_post_aggregates_mview_id ON post_aggregates_mview (id);

CREATE VIEW post_view AS
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
) AS us;

CREATE VIEW post_mview AS SELECT * FROM post_view;
