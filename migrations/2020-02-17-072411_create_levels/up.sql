-- Your SQL goes here
CREATE TABLE levels (
  term TEXT NOT NULL PRIMARY KEY,
  level INTEGER NOT NULL
);
CREATE INDEX levels_level_index ON levels(level);
