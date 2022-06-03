alter table community add column is_default_community boolean default false;
Update community set is_default_community = true where id = 2