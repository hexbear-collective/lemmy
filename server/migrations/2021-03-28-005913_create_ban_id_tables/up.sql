create table hexbear.ban_id (
    id          serial      primary key,
    created     timestamp   not null default now(),
    aliased_to  int         references hexbear.ban_id on update cascade on delete cascade
);

create table hexbear.user_ban_id (
    bid     int    references hexbear.ban_id on update cascade on delete cascade,
    uid     int    references user_ on update cascade on delete cascade,
    primary key (bid, uid)
);