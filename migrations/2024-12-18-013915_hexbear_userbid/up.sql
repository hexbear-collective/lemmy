DROP SCHEMA utils CASCADE;
DROP SCHEMA hexbear CASCADE;
DROP SCHEMA r CASCADE;

Create SCHEMA hexbear;
Create Table hexbear.user_cookie(
	cookie_uuid uuid PRIMARY KEY DEFAULT gen_random_uuid()
);

Create Table hexbear.user_cookie_local_users(
	cookie_uuid uuid REFERENCES hexbear.user_cookie ON DELETE CASCADE NOT NULL,
    local_user_id integer REFERENCES public.local_user ON DELETE CASCADE NOT NULL
);

CREATE INDEX idx_user_cookie_user_id ON hexbear.user_cookie_local_users (local_user_id);
ALTER TABLE hexbear.user_cookie_local_users ADD CONSTRAINT uq_cookie_user_id UNIQUE (cookie_uuid,local_user_id);