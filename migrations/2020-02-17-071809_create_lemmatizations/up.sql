-- Your SQL goes here
CREATE TABLE lemmatizations (
  id INTEGER PRIMARY KEY NOT NULL,
  source TEXT NOT NULL,
  target TEXT NOT NULL
);
CREATE INDEX lemmatizations_source_index ON lemmatizations(source);
