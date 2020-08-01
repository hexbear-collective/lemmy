create table user_tag (
  user_id       int references user_ on update cascade on delete cascade primary key,
  tags          jsonb not null
);

create table community_user_tag (
  user_id       int references user_ on update cascade on delete cascade primary key,
  community_id  int references community on update cascade on delete cascade,
  tags          jsonb not null
);
