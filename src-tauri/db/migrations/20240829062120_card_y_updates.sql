CREATE TABLE card_y_updates (
  id BLOB PRIMARY KEY,
  card_id BLOB REFERENCES cards(id) ON DELETE CASCADE NOT NULL,
  data BLOB NOT NULL,
  updated_at INTEGER NOT NULL DEFAULT (unixepoch('now', 'subsec') * 1000) ,
  is_checkpoint INTEGER NOT NULL DEFAULT 0,
  from_remote INTEGER NOT NULL
) STRICT;

CREATE INDEX card_y_updates$card_id_idx ON card_y_updates(card_id);

CREATE TRIGGER after_insert_card_y_updates$oplog
AFTER INSERT ON card_y_updates
FOR EACH ROW
BEGIN
  INSERT INTO oplog (primary_key, tablename, updated_at, is_deleted, is_synced, is_indexed)
  VALUES (
    NEW.id,
    "card_y_updates",
    NEW.updated_at,
    0,
    NEW.from_remote,
    0
  );
END;

CREATE TRIGGER after_update_card_y_updates$oplog
AFTER UPDATE ON card_y_updates
FOR EACH ROW
BEGIN
  INSERT INTO oplog (primary_key, tablename, updated_at, is_deleted, is_synced, is_indexed)
  VALUES (
    NEW.id,
    "card_y_updates",
    NEW.updated_at,
    0,
    NEW.from_remote,
    0
  );
END;

CREATE TRIGGER after_delete_card_y_updates$oplog
AFTER DELETE ON card_y_updates
FOR EACH ROW
BEGIN
  INSERT INTO oplog (primary_key, tablename, updated_at, is_deleted, is_synced, is_indexed)
  VALUES (
    OLD.id,
    "card_y_updates",
    unixepoch('now', 'subsec') * 1000,
    1, 
    OLD.from_remote,
    0
  );
END;
