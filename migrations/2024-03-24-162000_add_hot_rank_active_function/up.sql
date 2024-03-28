-- Replacing the old Hexbear hot_rank_active calculation with this function
CREATE OR REPLACE FUNCTION hot_rank_active (score numeric, published timestamp with time zone, newest_comment_time timestamp with time zone)
    RETURNS float
    AS $$
DECLARE
    hours_diff_published numeric := EXTRACT(EPOCH FROM (now() - published)) / 3600;
    adjusted_timestamp timestamp with time zone := (published
                +
                ('24:00:00'::interval
                    *
                    ( (1)::double precision
                      -
                      exp( (- 0.000012146493725346809)::double precision
                            *
                            date_part('epoch'::text,
                                (GREATEST(newest_comment_time, published) - published)
                            )
                      )
                    )
                )
              );
    hours_diff numeric := EXTRACT(EPOCH FROM (now() - adjusted_timestamp)) / 3600;
BEGIN
    -- 24 * 7 = 168, so after a week, it will default to 0.
    -- We use greatest(1,score+3) here and not greatest(2,score+2) to be fully true to the original
    -- (and also Hexbear has no downvotes, so it doesn't matter for us anyway)
    IF (hours_diff_published > 0 AND hours_diff_published < 168) THEN
        RETURN log(greatest (1, score + 3)) / power((hours_diff + 2), 1.8);
    ELSE
        -- if the post is from the future, set hot score to 0. otherwise you can game the post to
        -- always be on top even with only 1 vote by setting it to the future
        RETURN 0.0;
    END IF;
END;
$$
LANGUAGE plpgsql
IMMUTABLE PARALLEL SAFE;
