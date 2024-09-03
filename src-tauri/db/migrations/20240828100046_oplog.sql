CREATE TABLE oplog (
  rowid INTEGER PRIMARY KEY,
  primary_key BLOB NOT NULL,
  tablename TEXT NOT NULL,
  updated_at INTEGER NOT NULL,
  counter INTEGER NOT NULL DEFAULT 0,
  is_deleted INTEGER NOT NULL,
  status BLOB
) STRICT;

-- CREATE TRIGGER after_update_oplog$delete_when_synced_and_indexed
-- AFTER UPDATE ON oplog
-- FOR EACH ROW
-- WHEN
--   NEW.synced = 1 AND NEW.indexed = 1
-- BEGIN
--   DELETE FROM oplog
--   WHERE
--     primary_key = NEW.primary_key
--     AND updated_at <= NEW.updated_at
--     AND counter <= NEW.counter
--   ;
-- END;

CREATE TRIGGER after_update_oplog$update_counter
AFTER UPDATE ON oplog
FOR EACH ROW
WHEN
  EXISTS (
    SELECT 1
    FROM oplog
    WHERE updated_at = NEW.updated_at
    AND counter = NEW.counter
    AND rowid != NEW.rowid
    LIMIT 1
  )
BEGIN
  UPDATE oplog
  SET
    counter = (
      SELECT MAX(counter) + 1
      FROM oplog
      WHERE updated_at = NEW.updated_at
    )
  WHERE rowid = NEW.rowid;
END;
