create table hexbear.ban_id (
    id          int        primary key,
    created     timestamp  not null default now()
);

create table hexbear.user_ban_id (
    bid     int    references hexbear.ban_id on update cascade on delete cascade,
    uid     int    references user_ on update cascade on delete cascade,
    primary key (bid, uid)
);