CREATE TABLE pots (
  id BLOB PRIMARY KEY,
  name TEXT NOT NULL,
  owner BLOB REFERENCES users(id) ON DELETE CASCADE NOT NULL,
  created_at INTEGER NOT NULL DEFAULT (unixepoch('now', 'subsec') * 1000),
  updated_at INTEGER NOT NULL DEFAULT (unixepoch('now', 'subsec') * 1000)
) STRICT;
