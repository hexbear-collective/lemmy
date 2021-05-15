-- Your SQL goes here

alter table site add column autosubscribe_comms integer[] default '{}' not null;

drop view site_view;

create view site_view as
select s.*,
u.name as creator_name,
u.preferred_username as creator_preferred_username,
u.avatar as creator_avatar,
(select count(*) from user_) as number_of_users,
(select count(*) from post) as number_of_posts,
(select count(*) from comment) as number_of_comments,
(select count(*) from community) as number_of_communities
from site s
left join user_ u on s.creator_id = u.id;