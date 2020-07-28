create table user_tag (
  user_id       int references user_ on update cascade on delete cascade,
  tag_name      varchar(20),
  tag_value     varchar(80) not null,
  primary key   (user_id, tag_name)
);

create table community_user_tag (
  user_id       int references user_ on update cascade on delete cascade,
  community_id  int references community on update cascade on delete cascade,
  tag_name      varchar(20),
  tag_value     varchar(80) not null,
  primary key   (user_id, community_id, tag_name)
);
