ALTER TABLE post 
RENAME COLUMN stickied_community TO stickied;

DROP INDEX idx_post_stickied_local;

Alter table post
DROP COLUMN stickied_local 

ALTER TABLE post_aggregates 
RENAME COLUMN stickied_community TO stickied;

Alter table post_aggregates
DROP COLUMN stickied_local 