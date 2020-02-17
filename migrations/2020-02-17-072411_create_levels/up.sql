-- Your SQL goes here
CREATE TABLE levels (
  id INTEGER PRIMARY KEY NOT NULL,
  term TEXT NOT NULL,
  level INTEGER NOT NULL
);
CREATE INDEX levels_term_index ON levels(term);
