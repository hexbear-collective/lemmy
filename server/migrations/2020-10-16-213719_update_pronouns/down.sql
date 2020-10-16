-- Constraint: allowed_user_pronouns

ALTER TABLE public.user_tag DROP CONSTRAINT allowed_user_pronouns;

ALTER TABLE public.user_tag
ADD CONSTRAINT allowed_user_pronouns CHECK (('{he,him,she,her,they,them,any pronoun,doe,deer,e,em,eir,ey,fae,faer,xey,xem,undecided,none,use name}'::text[] @> regexp_split_to_array(tags->>'pronouns'::text, E'[,\/]')
OR (tags ->> 'pronouns'::text) IS NULL));