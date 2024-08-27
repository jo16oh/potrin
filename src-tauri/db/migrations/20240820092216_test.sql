PRAGMA foreign_keys = ON;
PRAGMA defer_foreign_keys = ON;

CREATE TABLE store (
  key TEXT PRIMARY KEY,
  value TEXT NOT NULL
) STRICT;

CREATE TABLE shape_info (
  tablename TEXT PRIMARY KEY,
  shape_id TEXT,
  offset TEXT,
  last_update INTEGER,
  counter INTEGER
) STRICT;

CREATE TABLE outlines (
  id BLOB PRIMARY KEY,
  parent BLOB REFERENCES outlines(id) ON DELETE CASCADE,
  text TEXT NOT NULL,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,
  counter INTEGER,
  origin INTEGER -- boolean
) STRICT;

CREATE INDEX outlines_parent_idx ON outlines(parent);
CREATE INDEX outlines_updated_at_idx ON outlines(updated_at);

CREATE TRIGGER after_insert_outlines_counter
AFTER INSERT ON outlines
FOR EACH ROW
WHEN
  EXISTS (
    SELECT 1 
    FROM outlines 
    WHERE updated_at = NEW.updated_at AND id != NEW.id 
    LIMIT 1
  )
BEGIN
    UPDATE outlines
    SET counter = (
        SELECT COUNT(*)
        FROM outlines
        WHERE updated_at = NEW.updated_at AND id != NEW.id
    )
    WHERE id = NEW.id;
END; 

CREATE TRIGGER after_update_outlines_counter
AFTER UPDATE ON outlines
FOR EACH ROW
WHEN
  EXISTS (
    SELECT 1 
    FROM outlines 
    WHERE updated_at = NEW.updated_at AND id != NEW.id 
    LIMIT 1
  )
BEGIN
    UPDATE outlines
    SET counter = (
        SELECT COUNT(*)
        FROM outlines
        WHERE updated_at = NEW.updated_at AND id != NEW.id
    )
    WHERE id = NEW.id;
END; 

-- CREATE TABLE oplog (
--   id TEXT PRIMARY KEY,
--   tablename TEXT NOT NULL,
--   data TEXT
-- ) STRICT;
--
-- CREATE TRIGGER outlines_log_inserts
-- AFTER INSERT ON outlines
-- FOR EACH ROW
-- BEGIN
--   INSERT INTO oplog (id, tablename, data)
--   VALUES (
--     NEW.id,
--     "outlines",
--     json_patch('{}', json_object('id', new."id", 'parent', new."parent", 'text', new."text", 'created_at', new."created_at", 'updated_at', new."updated_at", 'counter', new."counter"))
--   );
-- END;
--
-- CREATE TRIGGER outlines_log_updates
-- AFTER UPDATE ON outlines
-- FOR EACH ROW
-- BEGIN
--   INSERT INTO oplog (id, tablename, data)
--   VALUES (
--     NEW.id,
--     "outlines",
--     json_patch('{}', json_object('id', new."id", 'parent', new."parent", 'text', new."text", 'created_at', new."created_at", 'updated_at', new."updated_at", 'counter', new.'counter'))
--   )
--   ON CONFLICT(id)
--   DO UPDATE
--     SET
--       data = json_patch('{}', json_object('id', new."id", 'parent', new."parent", 'text', new."text", 'created_at', new."created_at", 'updated_at', new."updated_at", 'counter', new.'counter'))
--   ;
-- END;
