CREATE TABLE outlines (
  id UUID PRIMARY KEY,
  parent UUID REFERENCES outlines(id),
  text TEXT NOT NULL,
  created_at TIMESTAMP NOT NULL,
  updated_at TIMESTAMP NOT NULL
);

CREATE INDEX outlines_parent_idx ON outlines(parent);

