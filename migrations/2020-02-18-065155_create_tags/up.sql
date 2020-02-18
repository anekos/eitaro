-- Your SQL goes here
CREATE TABLE tags (
  id INTEGER PRIMARY KEY NOT NULL,
  term TEXT NOT NULL,
  tag TEXT NOT NULL
);
CREATE INDEX tags_term_index ON tags(term);
CREATE INDEX tags_tag_index ON tags(tag);
