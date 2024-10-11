CREATE TABLE versions (
  id BLOB PRIMARY KEY,
  pot_id BLOB REFERENCES pots(id) ON DELETE CASCADE NOT NULL,
  outline_id BLOB REFERENCES outlines(id) ON DELETE SET NULL,
  timestamp INTEGER NOT NULL
);

CREATE INDEX versions$outline_id_idx ON versions(outline_id);
