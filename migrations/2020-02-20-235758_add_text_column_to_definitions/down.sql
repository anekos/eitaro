-- This file should undo anything in `up.sql`
ALTER TABLE definitions RENAME TO definitions_temp;
CREATE TABLE definitions (
  id INTEGER PRIMARY KEY NOT NULL,
  term TEXT NOT NULL,
  definition TEXT NOT NULL
);
INSERT INTO definitions(id, term, definition) SELECT id, term, definition FROM definitions_temp;
DROP TABLE definitions_temp;
