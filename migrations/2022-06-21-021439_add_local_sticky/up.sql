ALTER TABLE post 
RENAME COLUMN stickied TO stickied_community;

Alter table post
ADD COLUMN stickied_local boolean NOT NULL Default false;

CREATE INDEX idx_post_stickied_local
ON post(stickied_local);

ALTER TABLE post_aggregates
RENAME COLUMN stickied TO stickied_community;

Alter table post_aggregates
ADD COLUMN stickied_local boolean NOT NULL Default false;