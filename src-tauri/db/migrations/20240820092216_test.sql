CREATE TABLE outlines (
  id TEXT PRIMARY KEY,
  parent TEXT REFERENCES outlines(id),
  text TEXT NOT NULL,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL
) STRICT;

CREATE INDEX outlines_parent_idx ON outlines(parent);

