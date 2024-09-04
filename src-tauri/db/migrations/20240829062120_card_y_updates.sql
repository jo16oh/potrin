CREATE TABLE card_y_updates (
  id BLOB PRIMARY KEY,
  card_id BLOB REFERENCES cards(id) ON DELETE CASCADE NOT NULL,
  data BLOB NOT NULL,
  updated_at INTEGER NOT NULL DEFAULT (unixepoch('now', 'subsec') * 1000) ,
  is_checkpoint INTEGER NOT NULL DEFAULT 0
) STRICT;

CREATE INDEX card_y_updates$card_id_idx ON card_y_updates(card_id);

CREATE TRIGGER after_insert_card_y_updates$oplog
AFTER INSERT ON card_y_updates
FOR EACH ROW
BEGIN
  INSERT INTO oplog (primary_key, tablename, updated_at, is_deleted, status)
  VALUES (
    NEW.id,
    "card_y_updates",
    NEW.updated_at,
    0,
    jsonb_object(
      'is_synced', jsonb(false)
    )
  );
END;

CREATE TRIGGER after_update_card_y_updates$oplog
AFTER UPDATE ON card_y_updates
FOR EACH ROW
BEGIN
  INSERT INTO oplog (primary_key, tablename, updated_at, is_deleted, status)
  VALUES (
    NEW.id,
    "card_y_updates",
    NEW.updated_at,
    0,
    jsonb_object(
      'is_synced', jsonb(false)
    )
  );
END;

CREATE TRIGGER after_delete_card_y_updates$oplog
AFTER DELETE ON card_y_updates
FOR EACH ROW
BEGIN
  INSERT INTO oplog (primary_key, tablename, updated_at, is_deleted, status)
  VALUES (
    OLD.id,
    "card_y_updates",
    unixepoch('now', 'subsec') * 1000,
    1, 
    jsonb_object(
      'is_synced', jsonb(false)
    )
  );
END;
