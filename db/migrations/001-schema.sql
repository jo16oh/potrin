CREATE TABLE IF NOT EXISTS users (
  id UUID PRIMARY KEY,
  name TEXT NOT NULL,
  created_at TIMESTAMP NOT NULL
);
ALTER TABLE users ENABLE ELECTRIC;

CREATE TABLE IF NOT EXISTS ydocs (
  id UUID PRIMARY KEY,
  type TEXT NOT NULL,
  last_materialized TEXT NOT NULL
);
ALTER TABLE ydocs ENABLE ELECTRIC;

CREATE TABLE IF NOT EXISTS ydoc_updates (
  id UUID PRIMARY KEY,
  ydoc_id UUID REFERENCES ydocs(id) NOT NULL ,
  data TEXT NOT NULL
);
ALTER TABLE ydoc_updates ENABLE ELECTRIC;

CREATE TABLE IF NOT EXISTS pots (
  id UUID PRIMARY KEY,
  name TEXT NOT NULL,
  owner UUID REFERENCES users(id),
  created_at TIMESTAMP NOT NULL
);
ALTER TABLE pots ENABLE ELECTRIC;

CREATE TABLE IF NOT EXISTS threads (
  id UUID PRIMARY KEY,
  parent_id UUID,
  fractional_index TEXT NOT NULL,
  title TEXT NOT NULL,
  created_at TIMESTAMP NOT NULL,
  updated_at TIMESTAMP NOT NULL,
  deleted BOOLEAN NOT NULL,
  author UUID REFERENCES users(id),
  pot_id UUID REFERENCES pots(id) ON DELETE CASCADE
);
CREATE INDEX threads_parent_id_idx ON threads(parent_id);
CREATE INDEX threads_fractional_index_idx ON threads(fractional_index);
ALTER TABLE threads ENABLE ELECTRIC;

CREATE TABLE IF NOT EXISTS cards (
  id UUID PRIMARY KEY,
  thread_id UUID NOT NULL,
  fractional_index TEXT NOT NULL,
  content TEXT NOT NULL,
  created_at TIMESTAMP NOT NULL,
  updated_at TIMESTAMP NOT NULL,
  deleted BOOLEAN NOT NULL,
  author UUID REFERENCES users(id),
  pot_id UUID REFERENCES pots(id) ON DELETE CASCADE,
  ydoc_id UUID REFERENCES ydocs(id) NOT NULL
);
CREATE INDEX cards_thread_id_idx ON cards(thread_id);
CREATE INDEX cards_fractional_index_idx ON cards(fractional_index);
ALTER TABLE cards ENABLE ELECTRIC;

