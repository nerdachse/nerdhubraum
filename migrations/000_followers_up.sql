CREATE TABLE IF NOT EXISTS followers
(
  id    INTEGER PRIMARY KEY   NOT NULL,
  name  CHAR(255)             NOT NULL,
  follower_id INTEGER         NOT NULL  UNIQUE,
  followed_at TEXT            NOT NULL
);
