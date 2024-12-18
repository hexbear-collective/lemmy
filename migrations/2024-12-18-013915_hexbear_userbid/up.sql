DROP SCHEMA utils CASCADE;
DROP SCHEMA hexbear CASCADE;

Create SCHEMA hexbear;

Create Table hexbear.user_cookie_person(
	cookie_uuid uuid NOT NULL,
    person_id integer REFERENCES public.person ON DELETE CASCADE NOT NULL
);

CREATE INDEX idx_user_cookie_person_id ON hexbear.user_cookie_person (person_id);
ALTER TABLE hexbear.user_cookie_person ADD CONSTRAINT uq_cookie_person_id UNIQUE (cookie_uuid,person_id);

Insert into hexbear.user_cookie_person
Select gen_random_uuid(), id
from public.person