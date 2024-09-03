CREATE TABLE outline_y_updates (
  id BLOB PRIMARY KEY,
  outline_id BLOB REFERENCES outlines(id) ON DELETE CASCADE NOT NULL,
  data BLOB NOT NULL,
  updated_at INTEGER NOT NULL DEFAULT (unixepoch('now', 'subsec') * 1000),
  is_checkpoint INTEGER NOT NULL DEFAULT 0,
  from_remote INTEGER NOT NULL
) STRICT;

CREATE INDEX outline_y_updates$outline_id_idx ON outline_y_updates(outline_id);

CREATE TRIGGER after_insert_outline_y_updates$oplog
AFTER INSERT ON outline_y_updates
FOR EACH ROW
BEGIN
  INSERT INTO oplog (primary_key, tablename, updated_at, is_deleted, is_synced, is_indexed)
  VALUES (
    NEW.id,
    "outline_y_updates",
    NEW.updated_at,
    1,
    NEW.from_remote,
    0
  );
END;

CREATE TRIGGER after_update_outline_y_updates$oplog
AFTER UPDATE ON outline_y_updates
FOR EACH ROW
BEGIN
  INSERT INTO oplog (primary_key, tablename, updated_at, is_deleted, is_synced, is_indexed)
  VALUES (
    NEW.id,
    "outline_y_updates",
    NEW.updated_at,
    1,
    NEW.from_remote,
    0
  );
END;

CREATE TRIGGER after_delete_outline_y_updates$oplog
AFTER DELETE ON outline_y_updates
FOR EACH ROW
BEGIN
  INSERT INTO oplog (primary_key, tablename, updated_at, is_deleted, is_synced, is_indexed)
  VALUES (
    OLD.id,
    "outline_y_updates",
    unixepoch('now', 'subsec') * 1000,
    1, 
    OLD.from_remote,
    0
  );
END;
