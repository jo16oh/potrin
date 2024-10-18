CREATE TABLE outline_links(
    rowid INTEGER PRIMARY KEY,
    id_from BLOB REFERENCES outlines(id) ON DELETE CASCADE,
    id_to BLOB REFERENCES outlines(id) ON DELETE CASCADE,
    UNIQUE (id_from, id_to)
) STRICT;

CREATE INDEX outline_links$from_idx ON outline_links(id_from);
CREATE INDEX outline_links$to_idx ON outline_links(id_to);

CREATE TRIGGER after_insert_outline_links
AFTER INSERT ON outline_links
FOR EACH ROW
BEGIN
  INSERT INTO oplog (primary_key, tablename, operation, updated_at, status)
  VALUES (
    CAST(NEW.rowid AS BLOB),
    "outline_links",
    "insert",
    unixepoch('now', 'subsec') * 1000,
    jsonb_object(
      'rowid', NEW.rowid,
      'is_synced', jsonb('false')
    )
  );
END;

CREATE TRIGGER after_update_outline_links
AFTER UPDATE ON outline_links
FOR EACH ROW
BEGIN
  SELECT RAISE(FAIL, "update is not allowed");
END;

CREATE TRIGGER after_delete_outline_links
AFTER DELETE ON outline_links
FOR EACH ROW
BEGIN
  INSERT INTO oplog (primary_key, tablename, operation, updated_at, status)
  VALUES (
    CAST(OLD.rowid AS BLOB),
    "outline_links",
    "delete",
    unixepoch('now', 'subsec') * 1000,
    jsonb_object(
      'rowid', OLD.rowid,
      'is_synced', jsonb('false')
    )
  );
END;



CREATE TABLE card_links (
    rowid INTEGER PRIMARY KEY,
    id_from BLOB REFERENCES cards(id) ON DELETE CASCADE,
    id_to BLOB REFERENCES outlines(id) ON DELETE CASCADE,
    UNIQUE (id_from, id_to)
) STRICT;

CREATE INDEX card_links$from_idx ON card_links(id_from);
CREATE INDEX card_links$to_idx ON card_links(id_to);

CREATE TRIGGER after_insert_card_links
AFTER INSERT ON card_links
FOR EACH ROW
BEGIN
  INSERT INTO oplog (primary_key, tablename, operation, updated_at, status)
  VALUES (
    CAST(NEW.rowid AS BLOB),
    "card_links",
    "insert",
    unixepoch('now', 'subsec') * 1000,
    jsonb_object(
      'rowid', NEW.rowid,
      'is_synced', jsonb('false')
    )
  );
END;

CREATE TRIGGER after_update_card_links
AFTER UPDATE ON card_links
FOR EACH ROW
BEGIN
  SELECT RAISE(FAIL, "update is not allowed");
END;

CREATE TRIGGER after_delete_card_links
AFTER DELETE ON card_links
FOR EACH ROW
BEGIN
  INSERT INTO oplog (primary_key, tablename, operation, updated_at, status)
  VALUES (
    CAST(OLD.rowid AS BLOB),
    "card_links",
    "delete",
    unixepoch('now', 'subsec') * 1000,
    jsonb_object(
      'rowid', OLD.rowid,
      'is_synced', jsonb('false')
    )
  );
END;



CREATE TABLE quotes (
    card_id BLOB REFERENCES cards(id) ON DELETE CASCADE PRIMARY KEY,
    quoted_card_id BLOB REFERENCES cards(id) ON DELETE SET NULL,
    version_id BLOB REFERENCES versions(id) ON DELETE RESTRICT NOT NULL
) STRICT;

CREATE INDEX quotes$quote_idx ON quotes(quoted_card_id);
CREATE INDEX quotes$version_id_idx ON quotes(version_id);

CREATE TRIGGER after_insert_quotes
AFTER INSERT ON quotes
FOR EACH ROW
BEGIN
  INSERT INTO oplog (primary_key, tablename, operation, updated_at, status)
  VALUES (
    NEW.card_id,
    "quotes",
    "insert",
    unixepoch('now', 'subsec') * 1000,
    jsonb_object(
      'is_synced', jsonb('false')
    )
  );
END;

CREATE TRIGGER after_update_quotes
AFTER INSERT ON quotes
FOR EACH ROW
BEGIN
  INSERT INTO oplog (primary_key, tablename, operation, updated_at, status)
  VALUES (
    NEW.card_id,
    "quotes",
    "update",
    unixepoch('now', 'subsec') * 1000,
    jsonb_object(
      'is_synced', jsonb('false')
    )
  );
END;

CREATE TRIGGER after_delete_quotes
AFTER DELETE ON quotes
FOR EACH ROW
BEGIN
  INSERT INTO oplog (primary_key, tablename, operation, updated_at, status)
  VALUES (
    OLD.card_id,
    "quotes",
    "delete",
    unixepoch('now', 'subsec') * 1000,
    jsonb_object(
      'is_synced', jsonb('false')
    )
  );
END;

