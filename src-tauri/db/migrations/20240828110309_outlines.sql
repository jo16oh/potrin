CREATE TABLE outlines (
  id BLOB PRIMARY KEY,
  pot_id BLOB REFERENCES pots(id) ON DELETE CASCADE, -- NOT NULL
  parent_id BLOB REFERENCES outlines(id) ON DELETE CASCADE,
  fractional_index TEXT,
  text TEXT,
  created_at INTEGER NOT NULL DEFAULT (unixepoch('now', 'subsec') * 1000),
  updated_at INTEGER NOT NULL DEFAULT (unixepoch('now', 'subsec') * 1000),
  is_deleted INTEGER NOT NULL DEFAULT 0,
  from_remote INTEGER NOT NULL
) STRICT;

CREATE INDEX outlines_parent_id_idx ON outlines(parent_id);
CREATE INDEX outlines_updated_at_idx ON outlines(updated_at);

CREATE TRIGGER after_insert_outlines$oplog
AFTER INSERT ON outlines
FOR EACH ROW
BEGIN
  INSERT INTO oplog (primary_key, tablename, updated_at, is_deleted, is_synced, is_indexed)
  VALUES (
    NEW.id,
    "outlines",
    NEW.updated_at,
    0,
    NEW.from_remote,
    0
  );
END;

CREATE TRIGGER after_update_outlines$oplog
AFTER UPDATE ON outlines
FOR EACH ROW
BEGIN
  INSERT INTO oplog (primary_key, tablename, updated_at, is_deleted, is_synced, is_indexed)
  VALUES (
    NEW.id,
    "outlines",
    NEW.updated_at,
    0,
    NEW.from_remote,
    0
  );
END;

CREATE TRIGGER after_delete_outlines$oplog
AFTER DELETE ON outlines
FOR EACH ROW
BEGIN
  INSERT INTO oplog (primary_key, tablename, updated_at, is_deleted, is_synced, is_indexed)
  VALUES (
    OLD.id,
    "outlines",
    unixepoch('now', 'subsec') * 1000,
    1, 
    OLD.from_remote,
    0
  );
END;
