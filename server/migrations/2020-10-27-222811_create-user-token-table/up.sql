create table hexbear.user_tokens (
  id          uuid      primary key default uuid_generate_v4(),
  user_id     int       references user_ on update cascade on delete cascade not null,
  token_hash  text      not null,
  created_at  timestamp not null default now(),
  expires_at  timestamp not null default now(),
  renewed_at  timestamp not null default now(),
  is_revoked  bool      not null default false
)
