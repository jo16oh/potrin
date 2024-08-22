CREATE TABLE outlines (
  id UUID PRIMARY KEY,
  parent UUID REFERENCES outlines(id),
  text TEXT NOT NULL
);

CREATE INDEX outlines_parent_idx ON outlines(parent);

