CREATE TABLE cards (
  id BLOB PRIMARY KEY,
  author BLOB REFERENCES users(id) ON DELETE CASCADE, -- NOT NULL
  outline_id BLOB REFERENCES outlines(id) ON DELETE CASCADE NOT NULL,
  fractional_index TEXT NOT NULL,
  text TEXT NOT NULL,
  last_materialized_hash BLOB,
  created_at INTEGER NOT NULL DEFAULT (unixepoch('now', 'subsec') * 1000),
  updated_at INTEGER NOT NULL DEFAULT (unixepoch('now', 'subsec') * 1000),
  is_deleted INTEGER NOT NULL DEFAULT 0
) STRICT;

CREATE INDEX cards_outline_id_idx ON cards(outline_id);
CREATE INDEX cards_updated_at_idx ON cards(updated_at);

CREATE TRIGGER after_insert_cards$oplog
AFTER INSERT ON cards
FOR EACH ROW
BEGIN
  INSERT INTO oplog (primary_key, tablename, updated_at, is_deleted, status)
  VALUES (
    NEW.id,
    "cards",
    NEW.updated_at,
    0,
    jsonb_object(
      'is_synced', jsonb('false'),
      'is_indexed', jsonb('false')
    )
  );
END;

CREATE TRIGGER after_update_cards$oplog
AFTER UPDATE ON cards
FOR EACH ROW
BEGIN
  INSERT INTO oplog (primary_key, tablename, updated_at, is_deleted, status)
  VALUES (
    NEW.id,
    "cards",
    NEW.updated_at,
    0,
    jsonb_object(
      'is_synced', jsonb('false'),
      'is_indexed', jsonb('false')
    )
  );
END;

CREATE TRIGGER after_delete_cards$oplog
AFTER DELETE ON cards
FOR EACH ROW
BEGIN
  INSERT INTO oplog (primary_key, tablename, updated_at, is_deleted, status)
  VALUES (
    OLD.id,
    "cards",
    unixepoch('now', 'subsec') * 1000,
    1, 
    jsonb_object(
      'is_synced', jsonb('false'),
      'is_indexed', jsonb('false')
    )
  );
END;
