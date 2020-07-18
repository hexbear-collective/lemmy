create extension "uuid-ossp";

create table comment_report (
  id            uuid      primary key default uuid_generate_v4(),
  time          timestamp not null default now(),
  reason        text,
  resolved      bool      not null default false,
  user_id       int       references user_ on update cascade on delete cascade not null,   -- user reporting comment
  comment_id    int       references comment on update cascade on delete cascade not null, -- comment being reported
  comment_text  text      not null,
  comment_time  timestamp not null,
  unique(comment_id, user_id) -- users should only be able to report a comment once
);

create table post_report (
  id            uuid      primary key default uuid_generate_v4(),
  time          timestamp not null default now(),
  reason        text,
  resolved      bool      not null default false,
  user_id       int       references user_ on update cascade on delete cascade not null, -- user reporting post
  post_id       int       references post on update cascade on delete cascade not null,  -- post being reported
  post_name	varchar(100) not null,
  post_url      text,
  post_body     text,
  post_time     timestamp not null,
  unique(post_id, user_id) -- users should only be able to report a post once
);

create view comment_report_view as
select cr.*,
c.post_id,
p.community_id
from comment_report cr
left join comment c on c.id = cr.comment_id
left join post p on p.id = c.post_id;

create view post_report_view as
select pr.*,
p.community_id
from post_report pr
left join post p on p.id = pr.post_id;
