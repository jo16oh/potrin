CREATE TABLE sync_status (
  pot_id BLOB REFERENCES pots(id) ON DELETE CASCADE,
  tablename TEXT NOT NULL,
  shape_id TEXT,
  offset TEXT,
  last_sent_timestamp INTEGER,
  PRIMARY KEY (pot_id, tablename)
);
