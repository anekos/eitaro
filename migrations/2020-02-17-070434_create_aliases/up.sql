-- Your SQL goes here
CREATE TABLE aliases (
  id INTEGER PRIMARY KEY NOT NULL,
  source TEXT NOT NULL,
  target TEXT NOT NULL
);
CREATE INDEX aliases_key_index ON aliases(source);
