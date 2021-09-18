drop index concurrently "idx_post_published" on public.post (published desc);

create or replace function hexbear.refresh_post()
  returns trigger
  language 'plpgsql'
as $BODY$
begin
IF (TG_OP = 'DELETE') THEN
    -- Update community number of posts
    update hexbear.community_stat set number_of_posts = number_of_posts - 1 where community_id = OLD.community_id;
    update hexbear.user_stat
    set
      number_of_posts = number_of_posts - 1
    where user_id = OLD.creator_id;

  ELSIF (TG_OP = 'UPDATE') THEN
  -- only hot ranks actually depend on a post update
    update hexbear.post_stat
    set
      hot_rank = hot_rank(coalesce(score, 1::bigint)::numeric, NEW.published),
      hot_rank_active = hot_rank(coalesce(score, 1::bigint)::numeric, 
        NEW.published + '24:00:00'::interval * (1::double precision - exp('-0.000012146493725346809'::numeric::double precision * 
        date_part('epoch'::text, GREATEST(newest_activity_time, NEW.published) - NEW.published
      ))))
    where post_id = NEW.id;

  ELSIF (TG_OP = 'INSERT') THEN

    insert into hexbear.post_stat (post_id, hot_rank, hot_rank_active, newest_activity_time)
    values (
      NEW.id,
      hot_rank(0::numeric, NEW.published),
      hot_rank(0::numeric, 
        NEW.published + '24:00:00'::interval * (1::double precision - exp('-0.000012146493725346809'::numeric::double precision * 
        date_part('epoch'::text, NEW.published - NEW.published)
      ))),
      NEW.published
    );

    -- Update that users number of posts
    update hexbear.user_stat
    set
      number_of_posts = number_of_posts + 1
    where user_id = NEW.creator_id;
  
    -- Update community number of posts
    update hexbear.community_stat set number_of_posts = number_of_posts + 1 where community_id = NEW.community_id;
  END IF;

  return null;
end $BODY$;