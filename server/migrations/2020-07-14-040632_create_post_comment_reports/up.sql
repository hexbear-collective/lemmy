create table comment_report (
  id            serial    primary key,
  comment_id    int       references comment on update cascade on delete cascade not null, -- comment being reported
  user_id       int       references user_ on update cascade on delete cascade not null,   -- user reporting comment
  reason        text,
  time          timestamp not null default now(),
  resolved      bool      not null default false,
  unique(comment_id, user_id) -- users should only be able to report a comment once
);

create table post_report (
  id            serial    primary key,
  post_id       int       references post on update cascade on delete cascade not null,  -- post being reported
  user_id       int       references user_ on update cascade on delete cascade not null, -- user reporting post
  reason        text,
  time          timestamp not null default now(),
  resolved      bool      not null default false,
  unique(post_id, user_id) -- users should only be able to report a post once
);
