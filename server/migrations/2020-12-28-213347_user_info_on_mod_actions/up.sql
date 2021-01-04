-- mod locking post
CREATE OR REPLACE VIEW hexbear.mod_lock_post_view
  AS
  SELECT
    mlp.id,
    mlp.mod_user_id,
    mlp.post_id,
    mlp.locked,
    mlp.when_,
    umod.name AS mod_user_name,
    uvictim.id AS other_user_id,
    uvictim.name AS other_user_name,
    p.name AS post_name,
    c.id AS community_id,
    c.name AS community_name
  FROM mod_lock_post mlp
     JOIN user_ umod ON umod.id = mlp.mod_user_id
     JOIN post p ON p.id = mlp.post_id
     JOIN community c ON c.id = p.community_id
     JOIN user_ uvictim ON uvictim.id = p.creator_id;


CREATE OR REPLACE VIEW hexbear.mod_remove_post_view
AS SELECT mrp.id,
    mrp.mod_user_id,
    mrp.post_id,
    mrp.reason,
    mrp.removed,
    mrp.when_,
    umod.name AS mod_user_name,
    uvictim.id AS other_user_id,
    uvictim.name AS other_user_name,
    p.name AS post_name,
    c.id AS community_id,
    c.name AS community_name
   FROM mod_remove_post mrp
     JOIN user_ umod ON umod.id = mrp.mod_user_id
     JOIN post p ON p.id = mrp.post_id
     JOIN community c ON c.id = p.community_id
     JOIN user_ uvictim ON uvictim.id = p.creator_id;


CREATE OR REPLACE VIEW hexbear.mod_sticky_post_view
AS SELECT msp.id,
    msp.mod_user_id,
    msp.post_id,
    msp.stickied,
    msp.when_,
    umod.name AS mod_user_name,
    uvictim.id AS other_user_id,
    uvictim.name AS other_user_name,
    p.name AS post_name,
    c.id AS community_id,
    c.name AS community_name
   FROM mod_sticky_post msp
     JOIN user_ umod ON umod.id = msp.mod_user_id
     JOIN post p ON p.id = msp.post_id
     JOIN community c ON c.id = p.community_id
     JOIN user_ uvictim ON uvictim.id = p.creator_id;