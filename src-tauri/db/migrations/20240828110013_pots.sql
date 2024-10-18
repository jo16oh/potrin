CREATE TABLE pots (
  id BLOB PRIMARY KEY,
  name TEXT NOT NULL,
  owner BLOB REFERENCES users(id) ON DELETE CASCADE NOT NULL,
  created_at INTEGER NOT NULL DEFAULT (unixepoch('now', 'subsec') * 1000),
  updated_at INTEGER NOT NULL DEFAULT (unixepoch('now', 'subsec') * 1000)
) STRICT;

CREATE TRIGGER after_insert_pots 
AFTER INSERT ON pots
FOR EACH ROW
BEGIN
  INSERT INTO oplog (primary_key, tablename, operation, updated_at, status)
    VALUES (
      NEW.id,
      "pots",
      "insert",
      NEW.updated_at,
      jsonb_object(
        'is_synced', jsonb('false')
      )
    );
END;

CREATE TRIGGER after_update_pots 
AFTER UPDATE ON pots
FOR EACH ROW
BEGIN
  INSERT INTO oplog (primary_key, tablename, operation, updated_at, status)
    VALUES (
      NEW.id,
      "pots",
      "update",
      NEW.updated_at,
      jsonb_object(
        'is_synced', jsonb('false')
      )
    );
END;

CREATE TRIGGER after_delete_pots 
AFTER DELETE ON pots
FOR EACH ROW
BEGIN
  INSERT INTO oplog (primary_key, tablename, operation, updated_at, status)
  VALUES (
    OLD.id,
    "pots",
    "delete",
    unixepoch('now', 'subsec') * 1000,
    jsonb_object(
      'is_synced', jsonb('false')
    )
  );
END;
