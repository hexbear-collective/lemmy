alter table user_ add column sitemod bool not null default false;

-- User
drop view user_view;
create view user_view as
select 
	u.id,
  u.actor_id,
	u.name,
	u.avatar,
	u.email,
	u.matrix_user_id,
  u.bio,
  u.local,
	u.admin,
  u.sitemod,
  u.banned,
  u.show_avatars,
  u.send_notifications_to_email,
  u.published,
  coalesce(pd.posts, 0) as number_of_posts,
  coalesce(pd.score, 0) as post_score,
  coalesce(cd.comments, 0) as number_of_comments,
  coalesce(cd.score, 0) as comment_score
from user_ u
left join (
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

drop table user_fast;
create table user_fast as select * from user_view;
alter table user_fast add primary key (id);
