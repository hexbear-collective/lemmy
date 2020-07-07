-- community_view
drop view community_aggregates_view cascade;
create view community_aggregates_view as
-- Now that there's public and private keys, you have to be explicit here
select c.id,
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
(select actor_id from user_ u where c.creator_id = u.id) as creator_actor_id,
(select local from user_ u where c.creator_id = u.id) as creator_local,
(select name from user_ u where c.creator_id = u.id) as creator_name,
(select avatar from user_ u where c.creator_id = u.id) as creator_avatar,
(select name from category ct where c.category_id = ct.id) as category_name,
(select count(*) from community_follower cf where cf.community_id = c.id) as number_of_subscribers,
(select count(*) from post p where p.community_id = c.id) as number_of_posts,
(select count(*) from comment co, post p where c.id = p.community_id and p.id = co.post_id) as number_of_comments,
hot_rank((select count(*) from community_follower cf where cf.community_id = c.id), c.published) as hot_rank
from community c;

create materialized view community_aggregates_mview as select * from community_aggregates_view;

create unique index idx_community_aggregates_mview_id on community_aggregates_mview (id);

create view community_view as
with all_community as
(
  select
  ca.*
  from community_aggregates_view ca
)

select
ac.*,
u.id as user_id,
(select cf.id::boolean from community_follower cf where u.id = cf.user_id and ac.id = cf.community_id) as subscribed
from user_ u
cross join all_community ac

union all

select 
ac.*,
null as user_id,
null as subscribed
from all_community ac
;

create view community_mview as
with all_community as
(
  select
  ca.*
  from community_aggregates_mview ca
)

select
ac.*,
u.id as user_id,
(select cf.id::boolean from community_follower cf where u.id = cf.user_id and ac.id = cf.community_id) as subscribed
from user_ u
cross join all_community ac

union all

select 
ac.*,
null as user_id,
null as subscribed
from all_community ac
;

-- community views
drop view community_moderator_view;
drop view community_follower_view;
drop view community_user_ban_view;

create view community_moderator_view as 
select *,
(select actor_id from user_ u where cm.user_id = u.id) as user_actor_id,
(select local from user_ u where cm.user_id = u.id) as user_local,
(select name from user_ u where cm.user_id = u.id) as user_name,
(select avatar from user_ u where cm.user_id = u.id),
(select actor_id from community c where cm.community_id = c.id) as community_actor_id,
(select local from community c where cm.community_id = c.id) as community_local,
(select name from community c where cm.community_id = c.id) as community_name
from community_moderator cm;

create view community_follower_view as 
select *,
(select actor_id from user_ u where cf.user_id = u.id) as user_actor_id,
(select local from user_ u where cf.user_id = u.id) as user_local,
(select name from user_ u where cf.user_id = u.id) as user_name,
(select avatar from user_ u where cf.user_id = u.id),
(select actor_id from community c where cf.community_id = c.id) as community_actor_id,
(select local from community c where cf.community_id = c.id) as community_local,
(select name from community c where cf.community_id = c.id) as community_name
from community_follower cf;

create view community_user_ban_view as 
select *,
(select actor_id from user_ u where cm.user_id = u.id) as user_actor_id,
(select local from user_ u where cm.user_id = u.id) as user_local,
(select name from user_ u where cm.user_id = u.id) as user_name,
(select avatar from user_ u where cm.user_id = u.id),
(select actor_id from community c where cm.community_id = c.id) as community_actor_id,
(select local from community c where cm.community_id = c.id) as community_local,
(select name from community c where cm.community_id = c.id) as community_name
from community_user_ban cm;
