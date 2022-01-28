-- This file should undo anything in `up.sql`
ALTER TABLE definitions RENAME TO definitions_temp;
CREATE TABLE definitions (
  id INTEGER PRIMARY KEY NOT NULL,
  term TEXT NOT NULL,
  definition TEXT NOT NULL,
  text TEXT NOT NULL
);
INSERT INTO definitions(id, term, definition, text) SELECT id, term, definition, text FROM definitions_temp;
DROP TABLE definitions_temp;
CREATE INDEX definitions_term_index ON definitions(term);
