CREATE TABLE outline_y_updates (
  id BLOB PRIMARY KEY,
  outline_id BLOB REFERENCES outlines(id) ON DELETE CASCADE NOT NULL,
  data BLOB NOT NULL,
  created_at INTEGER NOT NULL DEFAULT (unixepoch('now', 'subsec') * 1000)
) STRICT;

CREATE INDEX outline_y_updates$outline_id_idx ON outline_y_updates(outline_id);

CREATE TRIGGER after_insert_outline_y_updates$oplog
AFTER INSERT ON outline_y_updates
FOR EACH ROW
BEGIN
  INSERT INTO oplog (primary_key, tablename, operation, updated_at, status)
  VALUES (
    NEW.id,
    "outline_y_updates",
    "insert",
    NEW.created_at,
    jsonb_object(
      'is_synced', jsonb(false)
    )
  );
END;

CREATE TRIGGER after_update_outline_y_updates$oplog
AFTER UPDATE ON outline_y_updates
FOR EACH ROW
BEGIN
  SELECT RAISE(FAIL, 'update is not allowed');
END;

CREATE TRIGGER after_delete_outline_y_updates$oplog
AFTER DELETE ON outline_y_updates
FOR EACH ROW
BEGIN
  INSERT INTO oplog (primary_key, tablename, operation, updated_at, status)
  VALUES (
    OLD.id,
    "outline_y_updates",
    "delete",
    unixepoch('now', 'subsec') * 1000,
    jsonb_object(
      'is_synced', jsonb(false)
    )
  );
END;

CREATE TABLE version_outline_y_update (
  outline_y_update_id BLOB REFERENCES outline_y_updates(id) ON DELETE CASCADE NOT NULL,
  version_id BLOB REFERENCES versions(id) ON DELETE RESTRICT NOT NULL,
  PRIMARY KEY (outline_y_update_id, version_id)
);

CREATE INDEX version_outline_y_update$card_y_update_id ON version_outline_y_update(outline_y_update_id);

CREATE TABLE outline_y_updates_versions (
  rowid INTEGER PRIMARY KEY,
  version_id BLOB REFERENCES versions(id),
  y_update_id BLOB REFERENCES outline_y_updates(id),
  UNIQUE (version_id, y_update_id)
) STRICT;

CREATE INDEX outline_y_updates_versions$version_id ON outline_y_updates_versions(version_id);
CREATE INDEX outline_y_updates_versions$y_update_id ON outline_y_updates_versions(y_update_id);

CREATE TRIGGER after_insert_outline_y_updates_versions$oplog
AFTER INSERT ON outline_y_updates_versions
FOR EACH ROW
BEGIN
  INSERT INTO oplog (primary_key, tablename, operation, updated_at, status)
  VALUES (
    NEW.version_id,
    "outline_y_updates_versions",
    "insert",
    unixepoch('now', 'subsec') * 1000,
    jsonb_object(
      'rowid', NEW.rowid,
      'is_synced', jsonb(false)
    )
  );
END;

CREATE TRIGGER after_update_outline_y_updates_versions$oplog
AFTER UPDATE ON outline_y_updates_versions
FOR EACH ROW
BEGIN
  SELECT RAISE(FAIL, 'update is not allowed');
END;

CREATE TRIGGER after_delete_outline_y_updates_versions$oplog
AFTER DELETE ON outline_y_updates_versions
FOR EACH ROW
BEGIN
  INSERT INTO oplog (primary_key, tablename, operation, updated_at, status)
  VALUES (
    OLD.version_id,
    "outline_y_updates_versions",
    "delete",
    unixepoch('now', 'subsec') * 1000,
    jsonb_object(
      'rowid', OLD.rowid,
      'is_synced', jsonb(false)
    )
  );
END;
