-- Your SQL goes here
alter table user_
add column has_2fa boolean not null default false;

drop table user_fast;
drop view hexbear.user_view;
create view hexbear.user_view as
select
  u.id,
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
  u.published,
  coalesce(pd.posts, 0) as number_of_posts,
  coalesce(pd.score, 0) as post_score,
  coalesce(cd.comments, 0) as number_of_comments,
  coalesce(cd.score, 0) as comment_score
from user_ u
left join(
	select 
		p.creator_id as creator_id,
		count(distinct p.id) as posts,
		sum(pl.score) as score
	from post p
	join post_like pl on p.id = pl.post_id
	   group by p.creator_id
) pd on u.id = pd.creator_id
left join (
    select
        c.creator_id,
        count(distinct c.id) as comments,
        sum(cl.score) as score
    from comment c
    join comment_like cl on c.id = cl.comment_id
    group by c.creator_id
) cd on u.id = cd.creator_id;

create table user_fast as select * from hexbear.user_view;
alter table user_fast add primary key (id);