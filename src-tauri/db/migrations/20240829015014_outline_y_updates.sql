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
  INSERT INTO oplog (primary_key, tablename, updated_at, is_deleted, status)
  VALUES (
    NEW.id,
    "outline_y_updates",
    NEW.created_at,
    1,
    jsonb_object(
      'is_synced', jsonb(false)
    )
  );
END;

CREATE TRIGGER after_update_outline_y_updates$oplog
AFTER UPDATE ON outline_y_updates
FOR EACH ROW
BEGIN
  INSERT INTO oplog (primary_key, tablename, updated_at, is_deleted, status)
  VALUES (
    NEW.id,
    "outline_y_updates",
    NEW.created_at,
    1,
    jsonb_object(
      'is_synced', jsonb(false)
    )
  );
END;

CREATE TRIGGER after_delete_outline_y_updates$oplog
AFTER DELETE ON outline_y_updates
FOR EACH ROW
BEGIN
  INSERT INTO oplog (primary_key, tablename, updated_at, is_deleted, status)
  VALUES (
    OLD.id,
    "outline_y_updates",
    unixepoch('now', 'subsec') * 1000,
    1,
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
