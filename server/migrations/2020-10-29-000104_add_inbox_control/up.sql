-- Your SQL goes here
alter table user_
add column inbox_disabled boolean not null default false;

drop view hexbear.user_view;
CREATE OR REPLACE VIEW hexbear.user_view
 AS
 SELECT u.id,
    u.actor_id,
    u.name,
    u.preferred_username,
    u.avatar,
    u.banner,
    u.email,
    u.matrix_user_id,
    u.bio,
    u.local,
    u.admin,
    u.sitemod,
    u.banned,
    u.show_avatars,
    u.send_notifications_to_email,
    u.has_2fa,
    u.inbox_disabled,
    u.published,
    COALESCE(s.number_of_posts, 0)::bigint AS number_of_posts,
    COALESCE(s.post_score, 0)::bigint AS post_score,
    COALESCE(s.number_of_comments, 0)::bigint AS number_of_comments,
    COALESCE(s.comment_score, 0)::bigint AS comment_score
   FROM user_ u
     LEFT JOIN hexbear.user_stat s ON u.id = s.user_id;