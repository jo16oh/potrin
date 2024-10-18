CREATE TABLE card_y_updates (
  id BLOB PRIMARY KEY,
  card_id BLOB REFERENCES cards(id) ON DELETE CASCADE NOT NULL,
  data BLOB NOT NULL,
  created_at INTEGER NOT NULL DEFAULT (unixepoch('now', 'subsec') * 1000)
) STRICT;

CREATE INDEX card_y_updates$card_id_idx ON card_y_updates(card_id);

CREATE TRIGGER after_insert_card_y_updates$oplog
AFTER INSERT ON card_y_updates
FOR EACH ROW
BEGIN
  INSERT INTO oplog (primary_key, tablename, operation, updated_at, status)
  VALUES (
    NEW.id,
    "card_y_updates",
    "insert",
    NEW.created_at,
    jsonb_object(
      'is_synced', jsonb(false)
    )
  );
END;

CREATE TRIGGER after_update_card_y_updates$oplog
AFTER UPDATE ON card_y_updates
FOR EACH ROW
BEGIN
  SELECT RAISE(FAIL, 'update is not allowed');
END;

CREATE TRIGGER after_delete_card_y_updates$oplog
AFTER DELETE ON card_y_updates
FOR EACH ROW
BEGIN
  INSERT INTO oplog (primary_key, tablename, operation, updated_at, status)
  VALUES (
    OLD.id,
    "card_y_updates",
    "delete",
    unixepoch('now', 'subsec') * 1000,
    jsonb_object(
      'is_synced', jsonb(false)
    )
  );
END;

CREATE TABLE card_y_update_version (
  card_y_update_id BLOB REFERENCES card_y_updates(id) ON DELETE CASCADE NOT NULL,
  version_id BLOB REFERENCES versions(id) ON DELETE RESTRICT NOT NULL,
  PRIMARY KEY (card_y_update_id, version_id)
);

CREATE INDEX card_y_update_version$card_y_update_id ON card_y_update_version(card_y_update_id);

CREATE TABLE card_y_updates_versions (
  version_id BLOB REFERENCES versions(id),
  y_update_id BLOB REFERENCES card_y_updates(id),
  UNIQUE (version_id, y_update_id)
) STRICT;

CREATE INDEX card_y_updates_versions$version_id ON card_y_updates_versions(version_id);
CREATE INDEX card_y_updates_versions$y_update_id ON card_y_updates_versions(y_update_id);

CREATE TRIGGER after_insert_card_y_updates_versions$oplog
AFTER INSERT ON card_y_updates_versions
FOR EACH ROW
BEGIN
  INSERT INTO oplog (primary_key, tablename, operation, updated_at, status)
  VALUES (
    NEW.version_id,
    "card_y_updates_versions",
    "insert",
    unixepoch('now', 'subsec') * 1000,
    jsonb_object(
      'rowid', NEW.rowid,
      'is_synced', jsonb(false)
    )
  );
END;

CREATE TRIGGER after_update_card_y_updates_versions$oplog
AFTER UPDATE ON card_y_updates_versions
FOR EACH ROW
BEGIN
  SELECT RAISE(FAIL, 'update is not allowed');
END;

CREATE TRIGGER after_delete_card_y_updates_versions$oplog
AFTER DELETE ON card_y_updates_versions
FOR EACH ROW
BEGIN
  INSERT INTO oplog (primary_key, tablename, operation, updated_at, status)
  VALUES (
    OLD.version_id,
    "card_y_updates_versions",
    "delete",
    unixepoch('now', 'subsec') * 1000,
    jsonb_object(
      'rowid', OLD.rowid,
      'is_synced', jsonb(false)
    )
  );
END;
