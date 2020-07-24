-- Your SQL goes here
CREATE TABLE community_settings (
  id INT NOT NULL PRIMARY KEY,
  read_only BOOL NOT NULL,
  private BOOL NOT NULL,
  post_links BOOL NOT NULL,
  comment_images INT NOT NULL,
  published TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY (id)
    REFERENCES community(id)
    ON UPDATE CASCADE
    ON DELETE CASCADE
);

INSERT INTO community_settings (
  id,
  read_only,
  private,
  post_links,
  comment_images )
SELECT id,
  FALSE,
  FALSE,
  TRUE,
  1
FROM community;
