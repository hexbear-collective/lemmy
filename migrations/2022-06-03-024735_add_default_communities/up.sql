alter table community add column is_default_community boolean default false NOT NULL;
Update community set is_default_community = true where id = 2