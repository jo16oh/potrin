export default [
  {
    "statements": [
      "CREATE TABLE \"cards\" (\n  \"id\" TEXT NOT NULL,\n  \"thread\" TEXT,\n  \"fractional_index\" TEXT,\n  \"content\" TEXT,\n  \"created_at\" TEXT,\n  \"updated_at\" TEXT,\n  CONSTRAINT \"cards_pkey\" PRIMARY KEY (\"id\")\n) WITHOUT ROWID;\n",
      "CREATE INDEX \"cards_fractional_index_idx\" ON \"cards\" (\"fractional_index\" ASC);\n",
      "CREATE INDEX \"thread_idx\" ON \"cards\" (\"thread\" ASC);\n",
      "INSERT OR IGNORE INTO _electric_trigger_settings (namespace, tablename, flag) VALUES ('main', 'cards', 1);",
      "DROP TRIGGER IF EXISTS update_ensure_main_cards_primarykey;",
      "CREATE TRIGGER update_ensure_main_cards_primarykey\n  BEFORE UPDATE ON \"main\".\"cards\"\nBEGIN\n  SELECT\n    CASE\n      WHEN old.\"id\" != new.\"id\" THEN\n      \t\tRAISE (ABORT, 'cannot change the value of column id as it belongs to the primary key')\n    END;\nEND;",
      "DROP TRIGGER IF EXISTS insert_main_cards_into_oplog;",
      "CREATE TRIGGER insert_main_cards_into_oplog\n   AFTER INSERT ON \"main\".\"cards\"\n   WHEN 1 = (SELECT flag from _electric_trigger_settings WHERE namespace = 'main' AND tablename = 'cards')\nBEGIN\n  INSERT INTO _electric_oplog (namespace, tablename, optype, primaryKey, newRow, oldRow, timestamp)\n  VALUES ('main', 'cards', 'INSERT', json_patch('{}', json_object('id', new.\"id\")), json_object('content', new.\"content\", 'created_at', new.\"created_at\", 'fractional_index', new.\"fractional_index\", 'id', new.\"id\", 'thread', new.\"thread\", 'updated_at', new.\"updated_at\"), NULL, NULL);\nEND;",
      "DROP TRIGGER IF EXISTS update_main_cards_into_oplog;",
      "CREATE TRIGGER update_main_cards_into_oplog\n   AFTER UPDATE ON \"main\".\"cards\"\n   WHEN 1 = (SELECT flag from _electric_trigger_settings WHERE namespace = 'main' AND tablename = 'cards')\nBEGIN\n  INSERT INTO _electric_oplog (namespace, tablename, optype, primaryKey, newRow, oldRow, timestamp)\n  VALUES ('main', 'cards', 'UPDATE', json_patch('{}', json_object('id', new.\"id\")), json_object('content', new.\"content\", 'created_at', new.\"created_at\", 'fractional_index', new.\"fractional_index\", 'id', new.\"id\", 'thread', new.\"thread\", 'updated_at', new.\"updated_at\"), json_object('content', old.\"content\", 'created_at', old.\"created_at\", 'fractional_index', old.\"fractional_index\", 'id', old.\"id\", 'thread', old.\"thread\", 'updated_at', old.\"updated_at\"), NULL);\nEND;",
      "DROP TRIGGER IF EXISTS delete_main_cards_into_oplog;",
      "CREATE TRIGGER delete_main_cards_into_oplog\n   AFTER DELETE ON \"main\".\"cards\"\n   WHEN 1 = (SELECT flag from _electric_trigger_settings WHERE namespace = 'main' AND tablename = 'cards')\nBEGIN\n  INSERT INTO _electric_oplog (namespace, tablename, optype, primaryKey, newRow, oldRow, timestamp)\n  VALUES ('main', 'cards', 'DELETE', json_patch('{}', json_object('id', old.\"id\")), NULL, json_object('content', old.\"content\", 'created_at', old.\"created_at\", 'fractional_index', old.\"fractional_index\", 'id', old.\"id\", 'thread', old.\"thread\", 'updated_at', old.\"updated_at\"), NULL);\nEND;"
    ],
    "version": "20240609093426_047"
  },
  {
    "statements": [
      "CREATE TABLE \"threads\" (\n  \"id\" TEXT NOT NULL,\n  \"parent_thread\" TEXT,\n  \"fractional_index\" TEXT,\n  \"title\" TEXT,\n  \"created_at\" TEXT,\n  \"updated_at\" TEXT,\n  \"deleted\" INTEGER,\n  CONSTRAINT \"threads_pkey\" PRIMARY KEY (\"id\")\n) WITHOUT ROWID;\n",
      "CREATE INDEX \"parent_thread_idx\" ON \"threads\" (\"parent_thread\" ASC);\n",
      "CREATE INDEX \"threads_fractional_index_idx\" ON \"threads\" (\"fractional_index\" ASC);\n",
      "INSERT OR IGNORE INTO _electric_trigger_settings (namespace, tablename, flag) VALUES ('main', 'threads', 1);",
      "DROP TRIGGER IF EXISTS update_ensure_main_threads_primarykey;",
      "CREATE TRIGGER update_ensure_main_threads_primarykey\n  BEFORE UPDATE ON \"main\".\"threads\"\nBEGIN\n  SELECT\n    CASE\n      WHEN old.\"id\" != new.\"id\" THEN\n      \t\tRAISE (ABORT, 'cannot change the value of column id as it belongs to the primary key')\n    END;\nEND;",
      "DROP TRIGGER IF EXISTS insert_main_threads_into_oplog;",
      "CREATE TRIGGER insert_main_threads_into_oplog\n   AFTER INSERT ON \"main\".\"threads\"\n   WHEN 1 = (SELECT flag from _electric_trigger_settings WHERE namespace = 'main' AND tablename = 'threads')\nBEGIN\n  INSERT INTO _electric_oplog (namespace, tablename, optype, primaryKey, newRow, oldRow, timestamp)\n  VALUES ('main', 'threads', 'INSERT', json_patch('{}', json_object('id', new.\"id\")), json_object('created_at', new.\"created_at\", 'deleted', new.\"deleted\", 'fractional_index', new.\"fractional_index\", 'id', new.\"id\", 'parent_thread', new.\"parent_thread\", 'title', new.\"title\", 'updated_at', new.\"updated_at\"), NULL, NULL);\nEND;",
      "DROP TRIGGER IF EXISTS update_main_threads_into_oplog;",
      "CREATE TRIGGER update_main_threads_into_oplog\n   AFTER UPDATE ON \"main\".\"threads\"\n   WHEN 1 = (SELECT flag from _electric_trigger_settings WHERE namespace = 'main' AND tablename = 'threads')\nBEGIN\n  INSERT INTO _electric_oplog (namespace, tablename, optype, primaryKey, newRow, oldRow, timestamp)\n  VALUES ('main', 'threads', 'UPDATE', json_patch('{}', json_object('id', new.\"id\")), json_object('created_at', new.\"created_at\", 'deleted', new.\"deleted\", 'fractional_index', new.\"fractional_index\", 'id', new.\"id\", 'parent_thread', new.\"parent_thread\", 'title', new.\"title\", 'updated_at', new.\"updated_at\"), json_object('created_at', old.\"created_at\", 'deleted', old.\"deleted\", 'fractional_index', old.\"fractional_index\", 'id', old.\"id\", 'parent_thread', old.\"parent_thread\", 'title', old.\"title\", 'updated_at', old.\"updated_at\"), NULL);\nEND;",
      "DROP TRIGGER IF EXISTS delete_main_threads_into_oplog;",
      "CREATE TRIGGER delete_main_threads_into_oplog\n   AFTER DELETE ON \"main\".\"threads\"\n   WHEN 1 = (SELECT flag from _electric_trigger_settings WHERE namespace = 'main' AND tablename = 'threads')\nBEGIN\n  INSERT INTO _electric_oplog (namespace, tablename, optype, primaryKey, newRow, oldRow, timestamp)\n  VALUES ('main', 'threads', 'DELETE', json_patch('{}', json_object('id', old.\"id\")), NULL, json_object('created_at', old.\"created_at\", 'deleted', old.\"deleted\", 'fractional_index', old.\"fractional_index\", 'id', old.\"id\", 'parent_thread', old.\"parent_thread\", 'title', old.\"title\", 'updated_at', old.\"updated_at\"), NULL);\nEND;"
    ],
    "version": "20240609093426_161"
  }
]