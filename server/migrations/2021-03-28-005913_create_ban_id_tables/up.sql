create table hexbear.ban_id (
    id          uuid        primary key default uuid_generate_v4(),
    created     timestamp   not null default now(),
    aliased_to  uuid         references hexbear.ban_id on update cascade on delete cascade
);

create table hexbear.user_ban_id (
    bid     uuid   references hexbear.ban_id on update cascade on delete cascade,
    uid     int    references user_ on update cascade on delete cascade,
    primary key (bid, uid)
);