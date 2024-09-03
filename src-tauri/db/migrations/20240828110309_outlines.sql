CREATE TABLE outlines (
  id BLOB PRIMARY KEY,
  author BLOB REFERENCES users(id) ON DELETE SET NULL, 
  pot_id BLOB REFERENCES pots(id) ON DELETE CASCADE, -- NOT NULL
  parent_id BLOB REFERENCES outlines(id) ON DELETE CASCADE,
  fractional_index TEXT NOT NULL,
  text TEXT NOT NULL,
  last_materialized_hash BLOB,
  created_at INTEGER NOT NULL DEFAULT (unixepoch('now', 'subsec') * 1000),
  updated_at INTEGER NOT NULL DEFAULT (unixepoch('now', 'subsec') * 1000),
  is_deleted INTEGER NOT NULL DEFAULT 0
) STRICT;

CREATE INDEX outlines_parent_id_idx ON outlines(parent_id);
CREATE INDEX outlines_text_idx ON outlines(text);
CREATE INDEX outlines_updated_at_idx ON outlines(updated_at);

CREATE TRIGGER after_insert_outlines$oplog
AFTER INSERT ON outlines
FOR EACH ROW
BEGIN
  INSERT INTO oplog (primary_key, tablename, updated_at, is_deleted, status)
    VALUES (
      NEW.id,
      "outlines",
      NEW.updated_at,
      0,
      jsonb_object(
        'is_indexed', jsonb('false'),
        'is_conflicting', (
          CASE 
            WHEN EXISTS (
              SELECT 1 FROM outlines
              WHERE 
                text = NEW.text 
                AND id != NEW.id
                AND is_deleted = 0
                AND (parent_id = NEW.parent_id OR (parent_id IS NULL AND NEW.parent_id IS NULL))
            ) THEN jsonb('true')
            ELSE jsonb('false')
          END
        )
      )
    );
END;

CREATE TRIGGER after_update_outlines$oplog
AFTER UPDATE ON outlines
FOR EACH ROW
BEGIN
  INSERT INTO oplog (primary_key, tablename, updated_at, is_deleted, status)
  VALUES (
    NEW.id,
    "outlines",
    NEW.updated_at,
    0,
    jsonb_object(
      'is_indexed', jsonb('false'),
      'is_conflicting', (
        CASE 
          WHEN EXISTS (
            SELECT 1 FROM outlines
            WHERE 
              text = NEW.text 
              AND id != NEW.id
              AND is_deleted = 0
              AND (parent_id = NEW.parent_id OR (parent_id IS NULL AND NEW.parent_id IS NULL))
          ) THEN jsonb('true')
          ELSE jsonb('false')
        END
      )
    )
  );
END;

CREATE TRIGGER after_delete_outlines$oplog
AFTER DELETE ON outlines
FOR EACH ROW
BEGIN
  INSERT INTO oplog (primary_key, tablename, updated_at, is_deleted, status)
  VALUES (
    OLD.id,
    "outlines",
    unixepoch('now', 'subsec') * 1000,
    1, 
    jsonb_object(
      'is_indexed', jsonb('false'),
      'is_conflicting', (
        CASE 
          WHEN (
            SELECT COUNT(*)
            FROM outlines
            WHERE 
              text = OLD.text 
              AND id != OLD.id
              AND is_deleted = 0
          ) >= 2
          THEN jsonb('true')
          ELSE jsonb('false')
        END
      )
    )
  );
END;
