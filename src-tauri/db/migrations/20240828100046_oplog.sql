CREATE TABLE oplog (
  rowid INTEGER PRIMARY KEY,
  primary_key BLOB NOT NULL,
  tablename TEXT NOT NULL,
  updated_at INTEGER NOT NULL,
  counter INTEGER NOT NULL DEFAULT 0,
  is_deleted INTEGER NOT NULL,
  status BLOB
) STRICT;

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
