create or replace function hexbear.refresh_community()
    RETURNS trigger
    LANGUAGE 'plpgsql'
AS $BODY$
begin
  IF (TG_OP = 'UPDATE') THEN

    update hexbear.community_stat
    set
      hot_rank = hot_rank(number_of_subs::numeric, NEW.published)
    where community_id = NEW.id;
    -- Update user view due to owner changes
    delete from user_fast where id = NEW.creator_id;
    insert into user_fast select * from user_view where id = NEW.creator_id;
    
  -- TODO make sure this shows up in the users page ?
  ELSIF (TG_OP = 'INSERT') THEN
    insert into hexbear.community_stat (community_id, hot_rank)
    values (
      NEW.id,
      hot_rank(0::numeric, NEW.published)
    );
  END IF;

  return null;
end $BODY$;
