CREATE TABLE outlines (
  id UUID PRIMARY KEY,
  parent UUID REFERENCES outlines(id),
  text TEXT,
  created_at TEXT,
  updated_at TEXT
);

CREATE INDEX outlines_parent_idx ON outlines(parent);

