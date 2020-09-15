create or replace function hexbear.refresh_community()
    RETURNS trigger
    LANGUAGE 'plpgsql'
AS $BODY$
begin
  IF (TG_OP = 'UPDATE') THEN

    update hexbear.community_stat
    set
      hot_rank = hot_rank(number_of_subscribers::numeric, NEW.published)
    where community_id = NEW.id;

  ELSIF (TG_OP = 'INSERT') THEN
    insert into hexbear.community_stat (community_id, hot_rank)
    values (
      NEW.id,
      hot_rank(0::numeric, NEW.published)
    );
  END IF;

  return null;
end $BODY$;
