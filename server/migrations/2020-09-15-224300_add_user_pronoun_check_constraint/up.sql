update user_tag 
set tags = jsonb_set(tags, '{pronouns}', 'null')
where tags->>'pronouns' ~* '[[:<:]](?!(he|him|she|her|they|them|[any pronoun]))\w+[[:>:]]';

ALTER TABLE public.user_tag
ADD CONSTRAINT allowed_user_pronouns
CHECK ((tags->>'pronouns' !~* '[[:<:]](?!(he|him|she|her|they|them|[any pronoun]))\w+[[:>:]]') or tags->>'pronouns' is null);
