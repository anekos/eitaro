-- Your SQL goes here
CREATE TABLE definitions (
  id INTEGER PRIMARY KEY NOT NULL,
  term TEXT NOT NULL,
  definition TEXT NOT NULL
);
CREATE INDEX definitions_term_index ON definitions(term);
