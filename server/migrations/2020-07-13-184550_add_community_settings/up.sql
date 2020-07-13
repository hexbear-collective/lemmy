-- Your SQL goes here
CREATE TABLE community_settings (
  id SERIAL PRIMARY KEY,
  community_id INT NOT NULL,
  read_only BOOL NOT NULL,
  private BOOL NOT NULL,
  post_links BOOL NOT NULL,
  comment_images INT NOT NULL,
  published TIMESTAMP NOT NULL,
  FOREIGN KEY (community_id)
    REFERENCES community(id)
    ON UPDATE CASCADE
    ON DELETE CASCADE
);

INSERT INTO community_settings (
  community_id,
  read_only,
  private,
  post_links,
  comment_images,
  published )
SELECT id,
  FALSE,
  FALSE,
  TRUE,
  2,
  CURRENT_TIMESTAMP
FROM community;
