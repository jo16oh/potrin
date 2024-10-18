CREATE TABLE versions (
  id BLOB PRIMARY KEY,
  pot_id BLOB REFERENCES pots(id) ON DELETE CASCADE NOT NULL,
  timestamp INTEGER NOT NULL
);

CREATE TRIGGER after_insert_versions
AFTER INSERT ON versions
FOR EACH ROW
BEGIN
  INSERT INTO oplog (primary_key, tablename, operation, updated_at, status)
    VALUES (
      NEW.id,
      "versions",
      "insert",
      NEW.timestamp,
      jsonb_object(
        'is_synced', jsonb('false')
      )
    );
END;

CREATE TRIGGER after_update_versions
AFTER UPDATE ON versions
FOR EACH ROW
BEGIN
  SELECT RAISE(FAIL, "update is not allowed");
END;

CREATE TRIGGER after_delete_versions
AFTER DELETE ON versions
FOR EACH ROW
BEGIN
  INSERT INTO oplog (primary_key, tablename, operation, updated_at, status)
  VALUES (
    OLD.id,
    "versions",
    "delete",
    unixepoch('now', 'subsec') * 1000,
    jsonb_object(
      'is_synced', jsonb('false')
    )
  );
END;
