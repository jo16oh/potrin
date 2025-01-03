-- # SQLite Settings

PRAGMA foreign_keys = ON;
PRAGMA defer_foreign_keys = ON; -- Check foreign key consistancy when committing a transaction
PRAGMA journal_mode = 'WAL';
PRAGMA temp_store = 2; -- Store temp tables in memory
PRAGMA cache_size = -64000; -- Set the cache size to 64,000 KiB (negative value indicates kibibytes)




-- # Application State

CREATE TABLE kvs (
  id TEXT PRIMARY KEY,
  value BLOB NOT NULL
) STRICT;

CREATE TABLE workspaces (
  pot_id BLOB PRIMARY KEY,
  value BLOB NOT NULL
) STRICT;




-- # Operation Log

/*
  This table logs changes to other tables to ensure continuous synchronization 
  and indexing. Changes are recorded in a transaction by triggers. The `rowid` 
  is set as the primary key to uniquely identify each change, preventing any 
  changes from being missed during processing.
*/
CREATE TABLE operation_logs (
  rowid INTEGER PRIMARY KEY, 
  primary_key BLOB NOT NULL,
  tablename TEXT NOT NULL,
  operation TEXT NOT NULL,
  updated_at INTEGER NOT NULL,
  status BLOB
) STRICT;

CREATE INDEX operation_logs$primary_key ON operation_logs(primary_key);

/*
  Before inserting into operation_logs, delete any existing logs 
  with the same primary key, to save storage space.
*/
CREATE TRIGGER before_insert_operation_logs
BEFORE INSERT ON operation_logs
FOR EACH ROW
BEGIN
  DELETE FROM operation_logs
  WHERE 
    primary_key = NEW.primary_key 
    AND tablename = NEW.tablename;
END;




-- # Fundamental Tables

/*
  This table shouldn't be modified locally. All changes must be through the central database.
*/
CREATE TABLE users (
  id BLOB PRIMARY KEY,
  name TEXT NOT NULL,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL
) STRICT;

/*
  The `name` column can be updated locally, then synced to remote 
  by last-write-win strategy. 
*/
CREATE TABLE pots (
  id BLOB PRIMARY KEY,
  name TEXT NOT NULL,
  owner BLOB REFERENCES users(id) ON DELETE CASCADE,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL
) STRICT;

CREATE TRIGGER before_insert_pots 
BEFORE INSERT ON pots
FOR EACH ROW
BEGIN
  INSERT INTO operation_logs (primary_key, tablename, operation, updated_at)
    VALUES (
      NEW.id,
      "pots",
      "insert",
      NEW.updated_at
    );
END;

CREATE TRIGGER before_update_pots 
BEFORE UPDATE ON pots
FOR EACH ROW
BEGIN
  INSERT INTO operation_logs (primary_key, tablename, operation, updated_at)
    VALUES (
      NEW.id,
      "pots",
      "update",
      NEW.updated_at
    );
END;

CREATE TRIGGER before_delete_pots 
BEFORE DELETE ON pots
FOR EACH ROW
BEGIN
  INSERT INTO operation_logs (primary_key, tablename, operation, updated_at)
  VALUES (
    OLD.id,
    "pots",
    "delete",
    unixepoch('now', 'subsec') * 1000
  );
END;


-- sync_status
-- CREATE TABLE sync_status (
--   pot_id BLOB REFERENCES pots(id) ON DELETE CASCADE,
--   tablename TEXT NOT NULL,
--   shape_id TEXT,
--   shape_offset TEXT,
--   PRIMARY KEY (pot_id, tablename)
-- ) STRICT;




-- # Yjs Tables

/*
  These tables represent the Yjs data model. 
  All rows are immutable; UPDATE operation is not allowed.
  The `from_remote` column indicates the origin of the row,
  helping to determine which rows should be sent to remote.
*/

CREATE TABLE y_docs (
  id BLOB PRIMARY KEY,
  pot_id BLOB REFERENCES pots(id) ON DELETE CASCADE NOT NULL,
  author BLOB, -- implicitly referes to users(id)
  type TEXT NOT NULL,
  created_at INTEGER NOT NULL,
  from_remote INTEGER NOT NULL
) STRICT;

CREATE INDEX y_docs$created_at ON y_docs(created_at);

CREATE TABLE y_updates (
  id BLOB PRIMARY KEY,
  y_doc_id BLOB REFERENCES y_docs(id) ON DELETE CASCADE NOT NULL,
  data BLOB NOT NULL,
  version_id BLOB REFERENCES versions(id) ON DELETE RESTRICT,
  created_at INTEGER NOT NULL,
  from_remote INTEGER NOT NULL
) STRICT;

CREATE TRIGGER before_insert_y_updates
BEFORE INSERT ON y_updates
FOR EACH ROW
BEGIN
  INSERT INTO operation_logs (primary_key, tablename, operation, updated_at, status)
  VALUES (
    NEW.id,
    "y_updates",
    "insert",
    unixepoch('now', 'subsec') * 1000,
    jsonb_object(
      'from_remote', 
      CASE NEW.from_remote 
        WHEN 0 THEN jsonb('false')
        ELSE jsonb('true')
      END
    )
  );
END;

CREATE TRIGGER before_update_y_updates
BEFORE UPDATE ON y_updates
FOR EACH ROW
BEGIN
  SELECT RAISE(FAIL, 'update is not allowed');
END;

CREATE TRIGGER before_delete_y_updates
BEFORE DELETE ON y_updates
FOR EACH ROW
BEGIN
  INSERT INTO operation_logs (primary_key, tablename, operation, updated_at)
  VALUES (
    OLD.id,
    "y_updates",
    "delete",
    unixepoch('now', 'subsec') * 1000
  );
END;


/*
  Updates will be stored in this table until the change is commited, to restore updates if the app crashes.
*/
CREATE TABLE pending_y_updates (
  y_doc_id BLOB NOT NULL,
  data BLOB NOT NULL
) STRICT;

CREATE INDEX pending_y_updates$y_doc_id ON pending_y_updates(y_doc_id);

CREATE TRIGGER delete_pending_y_updates
BEFORE INSERT ON y_updates
FOR EACH ROW
BEGIN
  DELETE FROM pending_y_updates 
  WHERE 
    y_doc_id = NEW.y_doc_id
    AND from_remote = 0;
END;


/*
  Tag updates with a `version` to prevent merging.
  This enables restoring the state of a document at a specific point in time.
*/
CREATE TABLE versions (
  id BLOB PRIMARY KEY,
  pot_id BLOB REFERENCES pots(id) ON DELETE CASCADE NOT NULL,
  created_at INTEGER NOT NULL,
  from_remote INTEGER NOT NULL
) STRICT;

CREATE INDEX versions$created_at ON versions(created_at);

CREATE TRIGGER before_insert_versions
BEFORE INSERT ON versions
FOR EACH ROW
BEGIN
  INSERT INTO operation_logs (primary_key, tablename, operation, updated_at, status)
    VALUES (
      NEW.id,
      "versions",
      "insert",
      unixepoch('now', 'subsec') * 1000,
      jsonb_object(
        'from_remote', 
        CASE NEW.from_remote 
          WHEN 0 THEN jsonb('false')
          ELSE jsonb('true')
        END
      )
    );
END;

CREATE TRIGGER before_update_versions
BEFORE UPDATE ON versions
FOR EACH ROW
BEGIN
  SELECT RAISE(FAIL, 'update is not allowed');
END;

CREATE TRIGGER before_delete_versions
BEFORE DELETE ON versions
FOR EACH ROW
BEGIN
  INSERT INTO operation_logs (primary_key, tablename, operation, updated_at)
  VALUES (
    OLD.id,
    "versions",
    "delete",
    unixepoch('now', 'subsec') * 1000
  );
END;


/*
  This table holds each version's previous version. A `prev_version` represents the latest version 
  in the local db before inserting the new one. This table creates a singly linked list of updates. 
  By traveling this list, we can enumerate all updates that existed on the device when the version 
  was tagged, even if concurrent updates occured on other devices.
*/
CREATE TABLE prev_versions (
  id BLOB REFERENCES versions(id) ON DELETE CASCADE,
  prev_id BLOB REFERENCES versions(id) ON DELETE CASCADE,
  from_remote INTEGER NOT NULL,
  UNIQUE (id, prev_id)
) STRICT;

CREATE INDEX prev_versions$id ON prev_versions(id);
CREATE INDEX prev_versions$prev_id ON prev_versions(prev_id);


/*
  This table represents the latest versions in the pot.
*/
CREATE TABLE version_heads (
  id BLOB REFERENCES versions(id) ON DELETE CASCADE PRIMARY KEY
) STRICT;


/*
  Mark previous head as `prev_version` of the newly inserted update. 
*/
CREATE TRIGGER insert_prev_versions$before_insert_versions
BEFORE INSERT ON versions
FOR EACH ROW
WHEN NEW.from_remote = false
BEGIN
  INSERT OR IGNORE INTO prev_versions (id, prev_id, from_remote)
  SELECT NEW.id, version_heads.id, false
  FROM version_heads
  INNER JOIN versions ON version_heads.id = versions.id
  WHERE versions.pot_id = NEW.pot_id;
END;

/*
  Reconcile prev_version link when a version deleted
*/
CREATE TRIGGER insert_prev_versions$before_delete_versions
BEFORE DELETE ON versions
FOR EACH ROW
BEGIN
  INSERT OR IGNORE INTO prev_versions
  SELECT next.id AS id, prev.id AS prev_id, false AS from_remote
  FROM prev_versions AS current
  INNER JOIN prev_versions AS prev ON current.prev_id = prev.id
  INNER JOIN prev_versions AS next ON current.id = next.prev_id
  WHERE current.id = OLD.id;
END;

/*
  Mark a newly inserted update as the new head.
*/
CREATE TRIGGER insert_version_heads$after_insert_versions
AFTER INSERT ON versions
FOR EACH ROW
BEGIN
  INSERT INTO version_heads (id)
  SELECT id
  FROM versions
  WHERE
    id = NEW.id 
    AND NOT EXISTS (
      SELECT 1
      FROM prev_versions
      WHERE prev_id = versions.id
    );
END;

/*
  Unmark heads when they're marked as `prev_version`.
*/
CREATE TRIGGER delete_version_heads$after_insert_prev_versions
AFTER INSERT ON prev_versions
FOR EACH ROW
BEGIN
  DELETE FROM version_heads 
  WHERE id = NEW.prev_id;
END;





-- # Materialized Tables
/*
  These tables' records are "materialized" from y_updates. That means clients receive y_updates
  from remote and then "materialize" them into corresponding entities. All changes should be recorded 
  in y_updates. Materialized records are sent to remote using the last-write-win strategy.
*/

CREATE TABLE outlines (
  id BLOB REFERENCES y_docs(id) ON DELETE CASCADE PRIMARY KEY,
  parent_id BLOB REFERENCES outlines(id) ON DELETE CASCADE,
  fractional_index TEXT NOT NULL,
  doc TEXT NOT NULL,
  text TEXT,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,
  is_deleted INTEGER NOT NULL DEFAULT 0
) STRICT;

CREATE INDEX outlines$parent_id ON outlines(parent_id);
CREATE INDEX outlines$text ON outlines(text);
CREATE INDEX outlines$created_at ON outlines(created_at);
CREATE INDEX outlines$updated_at ON outlines(updated_at);

CREATE TRIGGER before_insert_outlines
BEFORE INSERT ON outlines
FOR EACH ROW
BEGIN
  INSERT INTO operation_logs (primary_key, tablename, operation, updated_at, status)
    VALUES (
      NEW.id,
      "outlines",
      "insert",
      NEW.updated_at,
      jsonb_object(
        'is_deleted', 
        CASE NEW.is_deleted 
          WHEN 0 THEN jsonb('false')
          ELSE jsonb('true')
        END
      )
    );
END;

CREATE TRIGGER before_update_outlines
BEFORE UPDATE ON outlines
FOR EACH ROW
BEGIN
  INSERT INTO operation_logs (primary_key, tablename, operation, updated_at, status)
  VALUES (
    NEW.id,
    "outlines",
    "update",
    NEW.updated_at,
    jsonb_object(
      'is_deleted', 
      CASE NEW.is_deleted 
        WHEN 0 THEN jsonb('false')
        ELSE jsonb('true')
      END
    )
  );
END;

CREATE TRIGGER before_delete_outlines
BEFORE DELETE ON outlines
FOR EACH ROW
BEGIN
  INSERT INTO operation_logs (primary_key, tablename, operation, updated_at, status)
  VALUES (
    OLD.id,
    "outlines",
    "delete",
    unixepoch('now', 'subsec') * 1000,
    jsonb_object(
      'pot_id',
      hex((
        SELECT pot_id
        FROM y_docs
        WHERE y_docs.id = OLD.id
      ))
    )
  );
END;


CREATE TABLE outline_links(
    rowid INTEGER PRIMARY KEY, -- set rowid as PK to specify the changed record in the operation_logs
    id_from BLOB REFERENCES outlines(id) ON DELETE CASCADE,
    id_to BLOB NOT NULL, -- implicitly referes to outlines(id), but is not constrained by FK to avoid inconsistency from y_updates
    UNIQUE (id_from, id_to)
) STRICT;

CREATE INDEX outline_links$id_from ON outline_links(id_from);
CREATE INDEX outline_links$id_to ON outline_links(id_to);

/*
  By separating path from the outlines table, 
  we exclude path changes from outline operation_logs, enabling lazy updates.
*/
CREATE TABLE outline_paths(
  outline_id BLOB REFERENCES outlines(id) ON DELETE CASCADE PRIMARY KEY,
  path BLOB NOT NULL
) STRICT;


CREATE TABLE cards (
  id BLOB REFERENCES y_docs(id) ON DELETE CASCADE PRIMARY KEY,
  outline_id BLOB REFERENCES outlines(id) ON DELETE CASCADE NOT NULL,
  fractional_index TEXT NOT NULL,
  doc TEXT NOT NULL,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,
  is_deleted INTEGER NOT NULL DEFAULT 0
) STRICT;

CREATE INDEX cards$outline_id ON cards(outline_id);
CREATE INDEX cards$created_at ON cards(created_at);
CREATE INDEX cards$updated_at ON cards(updated_at);

CREATE TRIGGER before_insert_cards
BEFORE INSERT ON cards
FOR EACH ROW
BEGIN
  INSERT INTO operation_logs (primary_key, tablename, operation, updated_at, status)
  VALUES (
    NEW.id,
    "cards",
    "insert",
    NEW.updated_at,
    jsonb_object(
      'is_deleted', 
      CASE NEW.is_deleted 
        WHEN 0 THEN jsonb('false')
        ELSE jsonb('true')
      END
    )
  );
END;

CREATE TRIGGER before_update_cards
BEFORE UPDATE ON cards
FOR EACH ROW
BEGIN
  INSERT INTO operation_logs (primary_key, tablename, operation, updated_at, status)
  VALUES (
    NEW.id,
    "update",
    "cards",
    NEW.updated_at,
    jsonb_object(
      'is_deleted', 
      CASE NEW.is_deleted 
        WHEN 0 THEN jsonb('false')
        ELSE jsonb('true')
      END
    )
  );
END;

CREATE TRIGGER before_delete_cards
BEFORE DELETE ON cards
FOR EACH ROW
BEGIN
  INSERT INTO operation_logs (primary_key, tablename, operation, updated_at, status)
  VALUES (
    OLD.id,
    "cards",
    "delete",
    unixepoch('now', 'subsec') * 1000,
    jsonb_object(
      'pot_id',
      hex((
        SELECT pot_id
        FROM y_docs
        WHERE y_docs.id = OLD.id
      ))
    )
  );
END;


CREATE TABLE card_links (
    rowid INTEGER PRIMARY KEY, -- set rowid as PK to specify the changed record in the operation_logs
    id_from BLOB REFERENCES cards(id) ON DELETE CASCADE NOT NULL,
    id_to BLOB NOT NULL, -- implicitly referes to outlines(id), but is not constrained by FK to avoid inconsistency from y_updates
    UNIQUE (id_from, id_to)
) STRICT;

CREATE INDEX card_links$id_from ON card_links(id_from);
CREATE INDEX card_links$id_to ON card_links(id_to);


CREATE TABLE quotes (
    card_id BLOB REFERENCES cards(id) ON DELETE CASCADE PRIMARY KEY,
    quote_id BLOB NOT NULL, -- implicitly referes to cards(id), but is not constrained by FK to avoid inconsistency from y_updates
    version_id BLOB NOT NULL -- same as above
) STRICT;

CREATE INDEX quotes$quote_id ON quotes(quote_id);
CREATE INDEX quotes$version_id ON quotes(version_id);

