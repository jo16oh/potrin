CREATE TABLE oplog (
  rowid INTEGER PRIMARY KEY,
  primary_key BLOB NOT NULL,
  tablename TEXT NOT NULL,
  operation TEXT NOT NULL,
  updated_at INTEGER NOT NULL,
  status BLOB
) STRICT;

CREATE INDEX oplog$primary_key ON oplog(primary_key);

CREATE TRIGGER before_insert_oplog
BEFORE INSERT ON oplog
FOR EACH ROW
BEGIN
  DELETE FROM oplog
  WHERE 
    primary_key = NEW.primary_key 
    AND tablename = NEW.tablename;
END;
